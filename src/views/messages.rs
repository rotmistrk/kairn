//! MessagesView — displays application message log.

use txv_core::prelude::*;

pub struct MessagesView {
    state: ViewState,
    messages: Vec<(String, MsgLevel)>,
    scroll: usize,
}

#[derive(Clone, Copy)]
pub enum MsgLevel {
    Info,
    Error,
}

impl Default for MessagesView {
    fn default() -> Self {
        Self::new()
    }
}

impl MessagesView {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            messages: Vec::new(),
            scroll: 0,
        }
    }

    pub fn push(&mut self, level: MsgLevel, msg: String) {
        self.messages.push((msg, level));
        self.state.dirty = true;
    }

    pub fn messages(&self) -> &[(String, MsgLevel)] {
        &self.messages
    }
}

impl View for MessagesView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        "Messages"
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let rows = b.h as usize;
        let start = if self.messages.len() > rows + self.scroll {
            self.messages.len() - rows - self.scroll
        } else {
            0
        };
        for row in 0..rows {
            let y = b.y + row as u16;
            if let Some((msg, level)) = self.messages.get(start + row) {
                let style = match level {
                    MsgLevel::Info => Style::default(),
                    MsgLevel::Error => Style {
                        fg: Color::Ansi(9),
                        ..Style::default()
                    },
                };
                surface.print_line(b.x, y, msg, b.w, style);
            } else {
                surface.print_line(b.x, y, "", b.w, Style::default());
            }
        }
    }

    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => {
                    if self.scroll < self.messages.len() {
                        self.scroll += 1;
                    }
                    self.state.dirty = true;
                    return HandleResult::Consumed;
                }
                KeyCode::Down => {
                    if self.scroll > 0 {
                        self.scroll -= 1;
                    }
                    self.state.dirty = true;
                    return HandleResult::Consumed;
                }
                _ => {}
            }
        }
        HandleResult::Ignored
    }
}
