//! InputLine — single-line text input with history and completion.

use txv_core::prelude::*;

pub struct InputLine {
    state: ViewState,
    pub text: String,
    pub cursor: usize,
    pub history: Vec<String>,
    history_pos: Option<usize>,
    pub completions: Vec<String>,
}

impl InputLine {
    pub fn new() -> Self {
        Self {
            state: ViewState::default(),
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_pos: None,
            completions: Vec::new(),
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
        self.cursor = self.text.len();
        self.state.dirty = true;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.state.dirty = true;
    }

    fn push_history(&mut self) {
        if !self.text.is_empty() {
            self.history.push(self.text.clone());
        }
        self.history_pos = None;
    }
}

impl Default for InputLine {
    fn default() -> Self {
        Self::new()
    }
}

impl View for InputLine {
    delegate_view_state!(state);

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let style = Style::default();
        surface.hline(b.x, b.y, b.w, ' ', style);
        // Compute visible window of text
        let w = b.w as usize;
        let start = if self.cursor >= w {
            self.cursor - w + 1
        } else {
            0
        };
        let visible: String = self.text.chars().skip(start).take(w).collect();
        surface.print(b.x, b.y, &visible, style);
        // Draw cursor
        let cx = (self.cursor - start) as u16;
        if cx < b.w {
            let ch = self.text.chars().nth(self.cursor).unwrap_or(' ');
            let cursor_style = Style {
                attrs: Attrs { reverse: true, ..Attrs::default() },
                ..Style::default()
            };
            surface.put(b.x + cx, b.y, ch, cursor_style);
        }
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match &key.code {
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += 1;
                self.state.dirty = true;
                HandleResult::Consumed
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Delete => {
                if self.cursor < self.text.len() {
                    self.text.remove(self.cursor);
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor += 1;
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Home => {
                self.cursor = 0;
                self.state.dirty = true;
                HandleResult::Consumed
            }
            KeyCode::End => {
                self.cursor = self.text.len();
                self.state.dirty = true;
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                self.push_history();
                queue.put_command(CM_OK, Some(Box::new(self.text.clone())));
                HandleResult::Consumed
            }
            KeyCode::Esc => {
                queue.put_command(CM_CANCEL, None);
                HandleResult::Consumed
            }
            KeyCode::Up => {
                // History navigation
                if !self.history.is_empty() {
                    let pos = match self.history_pos {
                        Some(p) => p.saturating_sub(1),
                        None => self.history.len() - 1,
                    };
                    self.history_pos = Some(pos);
                    self.text = self.history[pos].clone();
                    self.cursor = self.text.len();
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            KeyCode::Down => {
                if let Some(pos) = self.history_pos {
                    if pos + 1 < self.history.len() {
                        let next = pos + 1;
                        self.history_pos = Some(next);
                        self.text = self.history[next].clone();
                    } else {
                        self.history_pos = None;
                        self.text.clear();
                    }
                    self.cursor = self.text.len();
                    self.state.dirty = true;
                }
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
