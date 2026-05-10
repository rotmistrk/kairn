//! Search and ex command execution.

use super::motions;
use super::Editor;

// --- Search methods ---
impl Editor {
    pub(super) fn search_forward(&mut self, pattern: &str) {
        if pattern.is_empty() {
            return;
        }
        self.search_pattern = pattern.to_string();
        self.search_direction_forward = true;
        self.search_next();
    }

    pub(super) fn search_backward(&mut self, pattern: &str) {
        if pattern.is_empty() {
            return;
        }
        self.search_pattern = pattern.to_string();
        self.search_direction_forward = false;
        self.search_prev();
    }

    pub(super) fn search_next(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }
        let content = self.buffer.content();
        let start_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let search_from = start_offset + 1;
        if let Some(pos) = content[search_from..].find(&self.search_pattern) {
            let (l, c) = self.buffer.offset_to_line_col(search_from + pos);
            self.cursor_line = l;
            self.cursor_col = c;
        } else if let Some(pos) = content[..start_offset].find(&self.search_pattern) {
            let (l, c) = self.buffer.offset_to_line_col(pos);
            self.cursor_line = l;
            self.cursor_col = c;
            self.status = "search wrapped".to_string();
        }
    }

    pub(super) fn search_prev(&mut self) {
        if self.search_pattern.is_empty() {
            return;
        }
        let content = self.buffer.content();
        let start_offset = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        if let Some(pos) = content[..start_offset].rfind(&self.search_pattern) {
            let (l, c) = self.buffer.offset_to_line_col(pos);
            self.cursor_line = l;
            self.cursor_col = c;
        } else if let Some(pos) = content[start_offset + 1..].rfind(&self.search_pattern) {
            let (l, c) = self.buffer.offset_to_line_col(start_offset + 1 + pos);
            self.cursor_line = l;
            self.cursor_col = c;
            self.status = "search wrapped".to_string();
        }
    }

    pub(super) fn search_word(&mut self, forward: bool) {
        if let Some(word) = motions::word_at(&self.buffer, self.cursor_line, self.cursor_col) {
            self.search_pattern = word;
            self.search_direction_forward = forward;
            if forward {
                self.search_next();
            } else {
                self.search_prev();
            }
        }
    }
}

