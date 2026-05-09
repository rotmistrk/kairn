//! Text editing and clipboard methods.

use super::keymap::EditorMode;
use super::motions;
use super::Editor;

impl Editor {
    pub(super) fn enter_insert_after(&mut self) {
        self.buffer.begin_group();
        self.mode = EditorMode::Insert;
        let len = self.buffer.line_len(self.cursor_line);
        if self.cursor_col < len { self.cursor_col += 1; }
    }

    pub(super) fn exit_insert(&mut self) {
        self.buffer.end_group();
        self.mode = EditorMode::Normal;
        if self.cursor_col > 0 { self.cursor_col -= 1; }
    }

    pub(super) fn open_line_below(&mut self) {
        self.buffer.begin_group();
        self.mode = EditorMode::Insert;
        let line_len = self.buffer.line_len(self.cursor_line);
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, line_len) {
            self.buffer.insert(offset, "\n");
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub(super) fn open_line_above(&mut self) {
        self.buffer.begin_group();
        self.mode = EditorMode::Insert;
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
            self.buffer.insert(offset, "\n");
            self.cursor_col = 0;
        }
    }

    pub(super) fn insert_char(&mut self, ch: char) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
            if ch == '\t' {
                self.buffer.insert(offset, "    ");
                self.cursor_col += 4;
            } else {
                self.buffer.insert(offset, &ch.to_string());
                self.cursor_col += 1;
            }
        }
    }

    pub(super) fn insert_newline(&mut self) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
            self.buffer.insert(offset, "\n");
            self.cursor_line += 1;
            self.cursor_col = 0;
        }
    }

    pub(super) fn delete_char_forward(&mut self) {
        let line_len = self.buffer.line_len(self.cursor_line);
        if self.cursor_col < line_len {
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                let content = self.buffer.content();
                let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                self.buffer.delete(offset, offset + ch_len);
            }
        }
    }

    pub(super) fn delete_char_backward(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                let content = self.buffer.content();
                let ch_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
                self.buffer.delete(offset, offset + ch_len);
            }
        } else if self.cursor_line > 0 {
            let prev_len = self.buffer.line_len(self.cursor_line - 1);
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
                self.buffer.delete(offset - 1, offset);
                self.cursor_line -= 1;
                self.cursor_col = prev_len;
            }
        }
    }

    pub(super) fn delete_line(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        self.register = line;
        let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        let end = if self.cursor_line + 1 < self.buffer.line_count() {
            self.buffer.line_col_to_offset(self.cursor_line + 1, 0).unwrap_or(start)
        } else {
            self.buffer.content().len()
        };
        if start < end { self.buffer.delete(start, end); }
        if self.cursor_line >= self.buffer.line_count() && self.cursor_line > 0 {
            self.cursor_line -= 1;
        }
        self.clamp_col();
    }

    pub(super) fn delete_word(&mut self) {
        let start_offset = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col).unwrap_or(0);
        let (new_line, new_col) = motions::word_forward(&self.buffer, self.cursor_line, self.cursor_col);
        let end_offset = self.buffer.line_col_to_offset(new_line, new_col).unwrap_or(start_offset);
        if end_offset > start_offset {
            let content = self.buffer.content();
            self.register = content[start_offset..end_offset].to_string();
            self.buffer.delete(start_offset, end_offset);
            let (l, c) = self.buffer.offset_to_line_col(start_offset);
            self.cursor_line = l;
            self.cursor_col = c;
        }
    }

    pub(super) fn delete_word_backward(&mut self) {
        let end_offset = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col).unwrap_or(0);
        let (new_line, new_col) = motions::word_backward(&self.buffer, self.cursor_line, self.cursor_col);
        let start_offset = self.buffer.line_col_to_offset(new_line, new_col).unwrap_or(end_offset);
        if end_offset > start_offset {
            let content = self.buffer.content();
            self.register = content[start_offset..end_offset].to_string();
            self.buffer.delete(start_offset, end_offset);
            self.cursor_line = new_line;
            self.cursor_col = new_col;
        }
    }

    pub(super) fn delete_to_start(&mut self) {
        let end_offset = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col).unwrap_or(0);
        let start_offset = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        if end_offset > start_offset {
            let content = self.buffer.content();
            self.register = content[start_offset..end_offset].to_string();
            self.buffer.delete(start_offset, end_offset);
            self.cursor_col = 0;
        }
    }

    pub(super) fn delete_to_end(&mut self) {
        let start = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col).unwrap_or(0);
        let line_len = self.buffer.line_len(self.cursor_line);
        let end = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(start);
        if end > start {
            let content = self.buffer.content();
            self.register = content[start..end].to_string();
            self.buffer.delete(start, end);
        }
        self.clamp_col();
    }

    pub(super) fn change_line(&mut self) {
        self.buffer.begin_group();
        let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        let line_len = self.buffer.line_len(self.cursor_line);
        let end = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(start);
        if end > start { self.buffer.delete(start, end); }
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
    }

    pub(super) fn join_lines(&mut self) {
        if self.cursor_line + 1 >= self.buffer.line_count() { return; }
        let line_len = self.buffer.line_len(self.cursor_line);
        let end_offset = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
        self.buffer.delete(end_offset, end_offset + 1);
        let next_line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let after_join = &next_line[line_len..];
        let ws_count = after_join.chars().take_while(|c| c.is_whitespace()).count();
        if ws_count > 0 {
            let ws_start = self.buffer.line_col_to_offset(self.cursor_line, line_len).unwrap_or(0);
            let ws_end = self.buffer.line_col_to_offset(self.cursor_line, line_len + ws_count).unwrap_or(ws_start);
            self.buffer.delete(ws_start, ws_end);
        }
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, line_len) {
            self.buffer.insert(offset, " ");
        }
    }

    pub(super) fn toggle_case(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        if let Some(ch) = line.chars().nth(self.cursor_col) {
            let toggled = if ch.is_uppercase() {
                ch.to_lowercase().next().unwrap_or(ch)
            } else {
                ch.to_uppercase().next().unwrap_or(ch)
            };
            if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
                self.buffer.delete(offset, offset + ch.len_utf8());
                self.buffer.insert(offset, &toggled.to_string());
            }
            self.cursor_col += 1;
            self.clamp_col();
        }
    }

    pub(super) fn replace_char(&mut self, ch: char) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, self.cursor_col) {
            let content = self.buffer.content();
            let old_len = content[offset..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            self.buffer.delete(offset, offset + old_len);
            self.buffer.insert(offset, &ch.to_string());
        }
    }

    pub(super) fn indent_line(&mut self) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
            self.buffer.insert(offset, "    ");
            self.cursor_col += 4;
        }
    }

    pub(super) fn unindent_line(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let remove = if line.starts_with("    ") { 4 }
            else if line.starts_with('\t') { 1 }
            else { line.chars().take_while(|c| c.is_whitespace()).count().min(4) };
        if remove > 0 {
            let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
            let end = self.buffer.line_col_to_offset(self.cursor_line, remove).unwrap_or(start);
            self.buffer.delete(start, end);
            self.cursor_col = self.cursor_col.saturating_sub(remove);
        }
    }

}
