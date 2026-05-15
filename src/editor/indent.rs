//! Indentation operations for the editor.

use super::Editor;

impl Editor {
    pub(super) fn current_line_indent(&self) -> String {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        line.chars().take_while(|c| *c == ' ' || *c == '\t').collect()
    }

    pub(super) fn indent_line(&mut self) {
        if let Some(offset) = self.buffer.line_col_to_offset(self.cursor_line, 0) {
            self.buffer.insert(offset, "    ");
            self.cursor_col += 4;
        }
    }

    pub(super) fn unindent_line(&mut self) {
        let line = self.buffer.line(self.cursor_line).unwrap_or_default();
        let remove = if line.starts_with("    ") {
            4
        } else if line.starts_with('\t') {
            1
        } else {
            line.chars().take_while(|c| c.is_whitespace()).count().min(4)
        };
        if remove > 0 {
            let start = self.buffer.line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
            let end = self
                .buffer
                .line_col_to_offset(self.cursor_line, remove)
                .unwrap_or(start);
            self.buffer.delete(start, end);
            self.cursor_col = self.cursor_col.saturating_sub(remove);
        }
    }
}
