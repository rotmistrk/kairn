//! Dialog — modal dialog with message and OK/Cancel buttons.

use txv_core::prelude::*;

pub struct Dialog {
    state: ViewState,
    pub title_text: String,
    pub message: String,
    pub buttons: Vec<String>,
    pub focused_button: usize,
}

impl Dialog {
    pub fn new(title: impl Into<String>, message: impl Into<String>) -> Self {
        let mut s = Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            title_text: title.into(),
            message: message.into(),
            buttons: vec!["OK".into(), "Cancel".into()],
            focused_button: 0,
        };
        s.state.title = s.title_text.clone();
        s
    }

    pub fn set_buttons(&mut self, buttons: Vec<String>) {
        self.buttons = buttons;
        self.focused_button = 0;
        self.state.mark_dirty();
    }
}

impl View for Dialog {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let normal = Style::default();
        let border_style = Style {
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let btn_normal = Style::default();
        let btn_focused = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };

        // Fill background
        for row in 0..b.h {
            surface.hline(b.x, b.y + row, b.w, ' ', normal);
        }

        // Border
        surface.hline(b.x, b.y, b.w, '═', border_style);
        surface.hline(b.x, b.y + b.h.saturating_sub(1), b.w, '═', border_style);
        for row in 1..b.h.saturating_sub(1) {
            surface.put(b.x, b.y + row, '║', border_style);
            surface.put(b.x + b.w.saturating_sub(1), b.y + row, '║', border_style);
        }
        surface.put(b.x, b.y, '╔', border_style);
        surface.put(b.x + b.w.saturating_sub(1), b.y, '╗', border_style);
        surface.put(b.x, b.y + b.h.saturating_sub(1), '╚', border_style);
        surface.put(
            b.x + b.w.saturating_sub(1),
            b.y + b.h.saturating_sub(1),
            '╝',
            border_style,
        );

        // Title
        if !self.title_text.is_empty() {
            let title = format!(" {} ", self.title_text);
            let tx = b.x + 2;
            surface.print(tx, b.y, &title, border_style);
        }

        // Message
        let inner_w = b.w.saturating_sub(4) as usize;
        let msg_y = b.y + 2;
        for (i, line) in self.message.lines().enumerate() {
            let y = msg_y + i as u16;
            if y >= b.y + b.h.saturating_sub(2) {
                break;
            }
            let visible: String = line.chars().take(inner_w).collect();
            surface.print(b.x + 2, y, &visible, normal);
        }

        // Buttons at bottom
        let btn_y = b.y + b.h.saturating_sub(2);
        let total_btn_width: u16 = self.buttons.iter().map(|b| b.len() as u16 + 4).sum();
        let mut bx = b.x + (b.w.saturating_sub(total_btn_width)) / 2;
        for (i, btn) in self.buttons.iter().enumerate() {
            let style = if i == self.focused_button {
                btn_focused
            } else {
                btn_normal
            };
            let label = format!("[ {} ]", btn);
            surface.print(bx, btn_y, &label, style);
            bx += label.len() as u16 + 1;
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match key.code {
            KeyCode::Left | KeyCode::Tab => {
                if self.focused_button > 0 {
                    self.focused_button -= 1;
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Right | KeyCode::BackTab => {
                if self.focused_button + 1 < self.buttons.len() {
                    self.focused_button += 1;
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                let cmd = if self.focused_button == 0 {
                    CM_OK
                } else {
                    CM_CANCEL
                };
                queue.put_command(cmd, Some(Box::new(self.focused_button)));
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                queue.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Consumed,
        }
    }
}
