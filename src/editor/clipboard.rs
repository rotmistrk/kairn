//! Clipboard, yank, and multi-line operations.

use super::keymap::EditorMode;
use super::motions;
use super::Editor;

impl Editor {
    pub(super) fn paste_after(&mut self) {
        if !self.register.is_empty() {
            let line_len = self.buffer.line_len(self.cursor_line);
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, line_len) {
                let text = format!("\n{}", self.register);
                self.buffer.insert(offset, &text);
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        }
    }

    pub(super) fn paste_before(&mut self) {
        if !self.register.is_empty() {
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
                let text = format!("{}\n", self.register);
                self.buffer.insert(offset, &text);
                self.cursor_col = 0;
            }
        }
    }

    pub(super) fn yank_word(&mut self) {
        let start = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (nl, nc) = motions::word_forward(&self.buffer, self.cursor_line, self.cursor_col);
        let end = self.buffer.line_col_to_offset(nl, nc).unwrap_or(start);
        if end > start {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
        }
    }

    pub(super) fn yank_to_end(&mut self) {
        let start = self
            .buffer
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let line_len = self.buffer.line_len(self.cursor_line);
        let end = self
            .buffer
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
        }
    }

    pub(super) fn apply_set_option(&mut self, opt: &str) {
        match opt {
            "list" | "li" => self.options.list = true,
            "nolist" | "noli" => self.options.list = false,
            "number" | "nu" => self.options.number = true,
            "nonumber" | "nonu" => self.options.number = false,
            "wrap" => self.options.wrap = true,
            "nowrap" => self.options.wrap = false,
            _ => {
                self.status = format!("Unknown option: {opt}");
            }
        }
    }

    pub(super) fn yank_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buffer.line_count());
        let mut result = String::new();
        for i in self.cursor_line..end_line {
            result.push_str(&self.buffer.line(i).unwrap_or_default());
            result.push('\n');
        }
        self.register = result;
    }

    pub(super) fn delete_lines(&mut self, n: usize) {
        for _ in 0..n {
            self.delete_line();
        }
    }

    pub(super) fn change_lines(&mut self, n: usize) {
        for _ in 0..n {
            self.delete_line();
        }
        let offset = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        self.buffer.insert(offset, "\n");
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
    }

    pub(super) fn indent_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buffer.line_count());
        for line in self.cursor_line..end_line {
            if let Some(offset) = self.buffer.line_col_to_offset(line, 0) {
                self.buffer.insert(offset, "    ");
            }
        }
    }

    pub(super) fn unindent_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buffer.line_count());
        for line in self.cursor_line..end_line {
            let text = self.buffer.line(line).unwrap_or_default();
            let spaces = text.chars().take(4).take_while(|c| *c == ' ').count();
            if spaces > 0 {
                if let Some(offset) = self.buffer.line_col_to_offset(line, 0) {
                    self.buffer.delete(offset, offset + spaces);
                }
            }
        }
    }
}
