//! Menu — modal popup menu.

use txv_core::prelude::*;

pub struct MenuItem {
    pub label: String,
    pub command: CommandId,
    pub enabled: bool,
}

impl MenuItem {
    pub fn new(label: impl Into<String>, command: CommandId) -> Self {
        Self {
            label: label.into(),
            command,
            enabled: true,
        }
    }
}

pub struct Menu {
    state: ViewState,
    pub items: Vec<MenuItem>,
    pub cursor: usize,
}

impl Menu {
    pub fn new(items: Vec<MenuItem>) -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                modal: true,
                focusable: true,
                ..ViewOptions::default()
            }),
            items,
            cursor: 0,
        }
    }
}

impl View for Menu {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let normal = Style::default();
        let selected = Style {
            bg: Color::Ansi(4),
            attrs: Attrs {
                underline: true,
                ..Attrs::default()
            },
            ..Style::default()
        };
        let disabled = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };

        // Draw border
        surface.hline(b.x, b.y, b.w, '─', normal);
        surface.hline(b.x, b.y + b.h.saturating_sub(1), b.w, '─', normal);
        for row in 1..b.h.saturating_sub(1) {
            surface.put(b.x, b.y + row, '│', normal);
            surface.put(b.x + b.w.saturating_sub(1), b.y + row, '│', normal);
        }
        surface.put(b.x, b.y, '┌', normal);
        surface.put(b.x + b.w.saturating_sub(1), b.y, '┐', normal);
        surface.put(b.x, b.y + b.h.saturating_sub(1), '└', normal);
        surface.put(b.x + b.w.saturating_sub(1), b.y + b.h.saturating_sub(1), '┘', normal);

        // Draw items
        let inner_w = b.w.saturating_sub(2);
        for (i, item) in self.items.iter().enumerate() {
            let row = i as u16 + 1;
            if row >= b.h.saturating_sub(1) {
                break;
            }
            let y = b.y + row;
            let style = if !item.enabled {
                disabled
            } else if i == self.cursor {
                selected
            } else {
                normal
            };
            surface.hline(b.x + 1, y, inner_w, ' ', style);
            surface.print(b.x + 2, y, &item.label, style);
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Consumed; // modal captures all
        };
        match key.code {
            KeyCode::Up => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if self.cursor + 1 < self.items.len() {
                    self.cursor += 1;
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                if let Some(item) = self.items.get(self.cursor) {
                    if item.enabled {
                        queue.put_command(item.command, None);
                    }
                }
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                queue.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Consumed, // modal swallows all keys
        }
    }
}
