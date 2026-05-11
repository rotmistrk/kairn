//! InlineEditor — generic inline text editing for row-based widgets.

use txv_core::prelude::*;

/// Result of handling a key in the inline editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineEditResult {
    /// Key consumed, editing continues.
    Continue,
    /// User pressed Enter — commit the buffer.
    Commit(String),
    /// User pressed Escape — cancel editing.
    Cancel,
}

/// Delegate trait for inline editing behavior.
pub trait InlineEditDelegate: Send + 'static {
    /// Can the item at this visible row be edited?
    fn can_edit(&self, row: usize) -> bool;
    /// Validate in-progress text. None = valid, Some(msg) = error.
    fn validate(&self, row: usize, text: &str) -> Option<String>;
    /// Tab-completion candidates. Empty = no completions.
    fn complete(&self, _row: usize, _text: &str) -> Vec<String> {
        vec![]
    }
    /// Commit the edit. Called on Enter when validate returns None.
    fn commit(&mut self, row: usize, text: String);
}

/// Inline single-line editor embedded in a row.
pub struct InlineEditor {
    pub row: usize,
    pub buffer: String,
    pub cursor: usize,
}

impl InlineEditor {
    pub fn new(row: usize, initial_text: &str) -> Self {
        let cursor = initial_text.len();
        Self {
            row,
            buffer: initial_text.to_owned(),
            cursor,
        }
    }

    /// Handle a key event. Returns the editing result.
    pub fn handle_key(&mut self, key: &KeyEvent) -> InlineEditResult {
        match key.code {
            KeyCode::Enter => InlineEditResult::Commit(self.buffer.clone()),
            KeyCode::Esc => InlineEditResult::Cancel,
            KeyCode::Char(ch) => {
                self.insert_char(ch);
                InlineEditResult::Continue
            }
            KeyCode::Backspace => {
                self.delete_before();
                InlineEditResult::Continue
            }
            KeyCode::Delete => {
                self.delete_at();
                InlineEditResult::Continue
            }
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
                InlineEditResult::Continue
            }
            KeyCode::Right => {
                if self.cursor < self.buffer.len() {
                    self.cursor += 1;
                }
                InlineEditResult::Continue
            }
            KeyCode::Home => {
                self.cursor = 0;
                InlineEditResult::Continue
            }
            KeyCode::End => {
                self.cursor = self.buffer.len();
                InlineEditResult::Continue
            }
            _ => InlineEditResult::Continue,
        }
    }

    /// Draw the editor at the given position on the surface.
    pub fn draw(&self, surface: &mut Surface, x: u16, y: u16, width: u16, style: Style) {
        surface.hline(x, y, width, ' ', style);
        let visible = if self.buffer.len() > width as usize {
            &self.buffer[..width as usize]
        } else {
            &self.buffer
        };
        surface.print(x, y, visible, style);
        // Draw cursor
        let cx = x + self.cursor as u16;
        if cx < x + width {
            let ch = self.buffer.chars().nth(self.cursor).unwrap_or(' ');
            let cursor_style = Style {
                fg: style.bg,
                bg: style.fg,
                ..style
            };
            surface.put(cx, y, ch, cursor_style);
        }
    }

    /// Apply tab completion: cycle through candidates.
    pub fn apply_completion(&mut self, candidates: &[String], direction: i32) {
        if candidates.is_empty() {
            return;
        }
        let idx = candidates
            .iter()
            .position(|c| c == &self.buffer)
            .map(|i| {
                if direction > 0 {
                    (i + 1) % candidates.len()
                } else {
                    (i + candidates.len() - 1) % candidates.len()
                }
            })
            .unwrap_or(0);
        if let Some(text) = candidates.get(idx) {
            self.buffer = text.clone();
            self.cursor = self.buffer.len();
        }
    }

    fn insert_char(&mut self, ch: char) {
        self.buffer.insert(self.cursor, ch);
        self.cursor += ch.len_utf8();
    }

    fn delete_before(&mut self) {
        if self.cursor > 0 {
            let prev = self.buffer[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.buffer.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    fn delete_at(&mut self) {
        if self.cursor < self.buffer.len() {
            let next = self.buffer[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.buffer.len());
            self.buffer.drain(self.cursor..next);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent {
            code,
            modifiers: KeyMod::default(),
        }
    }

    #[test]
    fn insert_and_commit() {
        let mut ed = InlineEditor::new(0, "");
        assert_eq!(ed.handle_key(&key(KeyCode::Char('h'))), InlineEditResult::Continue);
        assert_eq!(ed.handle_key(&key(KeyCode::Char('i'))), InlineEditResult::Continue);
        assert_eq!(ed.buffer, "hi");
        assert_eq!(ed.cursor, 2);
        assert_eq!(
            ed.handle_key(&key(KeyCode::Enter)),
            InlineEditResult::Commit("hi".to_owned())
        );
    }

    #[test]
    fn cancel() {
        let mut ed = InlineEditor::new(0, "text");
        assert_eq!(ed.handle_key(&key(KeyCode::Esc)), InlineEditResult::Cancel);
    }

    #[test]
    fn backspace_and_delete() {
        let mut ed = InlineEditor::new(0, "abc");
        // cursor at end (3)
        ed.handle_key(&key(KeyCode::Backspace));
        assert_eq!(ed.buffer, "ab");
        assert_eq!(ed.cursor, 2);
        ed.handle_key(&key(KeyCode::Home));
        ed.handle_key(&key(KeyCode::Delete));
        assert_eq!(ed.buffer, "b");
        assert_eq!(ed.cursor, 0);
    }

    #[test]
    fn navigation() {
        let mut ed = InlineEditor::new(0, "hello");
        ed.handle_key(&key(KeyCode::Home));
        assert_eq!(ed.cursor, 0);
        ed.handle_key(&key(KeyCode::Right));
        assert_eq!(ed.cursor, 1);
        ed.handle_key(&key(KeyCode::End));
        assert_eq!(ed.cursor, 5);
        ed.handle_key(&key(KeyCode::Left));
        assert_eq!(ed.cursor, 4);
    }

    #[test]
    fn tab_completion() {
        let mut ed = InlineEditor::new(0, "");
        let candidates = vec!["alpha".to_owned(), "beta".to_owned(), "gamma".to_owned()];
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "alpha");
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "beta");
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "gamma");
        ed.apply_completion(&candidates, 1);
        assert_eq!(ed.buffer, "alpha");
    }
}
