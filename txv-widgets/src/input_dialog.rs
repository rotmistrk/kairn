//! InputDialog — modal dialog with a single-line text input.
//! Emits CM_OK with the entered text, or CM_CANCEL on Esc.

use txv_core::prelude::*;

/// A modal prompt dialog with a title and single-line input.
pub struct InputDialog {
    state: ViewState,
    title_text: String,
    text: String,
    cursor: usize,
}

impl InputDialog {
    pub fn new(title: impl Into<String>) -> Self {
        let mut s = Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            title_text: title.into(),
            text: String::new(),
            cursor: 0,
        };
        s.state.title = s.title_text.clone();
        s
    }

    /// Get the entered text.
    pub fn text(&self) -> &str {
        &self.text
    }
}

impl View for InputDialog {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let normal = Style::default();
        let border = Style {
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        // Fill background
        for row in 0..b.h {
            surface.hline(b.x, b.y + row, b.w, ' ', normal);
        }
        // Border
        surface.hline(b.x, b.y, b.w, '═', border);
        surface.hline(b.x, b.y + b.h.saturating_sub(1), b.w, '═', border);
        for row in 1..b.h.saturating_sub(1) {
            surface.put(b.x, b.y + row, '║', border);
            surface.put(b.x + b.w.saturating_sub(1), b.y + row, '║', border);
        }
        surface.put(b.x, b.y, '╔', border);
        surface.put(b.x + b.w.saturating_sub(1), b.y, '╗', border);
        surface.put(b.x, b.y + b.h.saturating_sub(1), '╚', border);
        surface.put(b.x + b.w.saturating_sub(1), b.y + b.h.saturating_sub(1), '╝', border);
        // Title
        if !self.title_text.is_empty() {
            let title = format!(" {} ", self.title_text);
            surface.print(b.x + 2, b.y, &title, border);
        }
        // Input line
        let inner_w = b.w.saturating_sub(4) as usize;
        let input_y = b.y + 2;
        let start = if self.cursor >= inner_w {
            self.cursor - inner_w + 1
        } else {
            0
        };
        let visible: String = self.text.chars().skip(start).take(inner_w).collect();
        surface.print(b.x + 2, input_y, &visible, normal);
        // Cursor
        let cx = (self.cursor - start) as u16;
        if cx < inner_w as u16 {
            let ch = self.text.chars().nth(self.cursor).unwrap_or(' ');
            let cursor_style = Style {
                attrs: Attrs {
                    reverse: true,
                    ..Attrs::default()
                },
                ..Style::default()
            };
            surface.put(b.x + 2 + cx, input_y, ch, cursor_style);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match &key.code {
            KeyCode::Enter => {
                queue.put_command(CM_OK, Some(Box::new(self.text.clone())));
            }
            KeyCode::Esc => {
                queue.put_command(CM_CANCEL, None);
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                    self.state.dirty = true;
                }
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.state.dirty = true;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor += 1;
                    self.state.dirty = true;
                }
            }
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += 1;
                self.state.dirty = true;
            }
            _ => {}
        }
        HandleResult::Consumed
    }
}
