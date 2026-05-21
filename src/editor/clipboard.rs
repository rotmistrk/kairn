//! Clipboard, yank, and multi-line operations.

use crate::settings::CursorStyle;

use super::keymap::EditorMode;
use super::motions;
use super::Editor;

impl Editor {
    /// Set the yank register and copy to system clipboard via OSC 52.
    pub fn yank(&mut self, text: String) {
        self.register = text.clone();
        if let Err(e) = crate::clipboard::copy_to_clipboard(&text) {
            self.status = format!("clipboard: {e}");
        }
    }

    pub(super) fn paste_after(&mut self) {
        if !self.register.is_empty() {
            let line_len = self.buf().line_len(self.cursor_line);
            let offset = self.buf().line_col_to_offset(self.cursor_line, line_len);
            if let Some(offset) = offset {
                let text = format!("\n{}", self.register);
                self.buf().insert(offset, &text);
                self.cursor_line += 1;
                self.cursor_col = 0;
            }
        }
    }

    pub(super) fn paste_before(&mut self) {
        if !self.register.is_empty() {
            let offset = self.buf().line_col_to_offset(self.cursor_line, 0);
            if let Some(offset) = offset {
                let text = format!("{}\n", self.register);
                self.buf().insert(offset, &text);
                self.cursor_col = 0;
            }
        }
    }

    pub(super) fn yank_word(&mut self) {
        let start = self
            .buf()
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let (nl, nc) = motions::word_forward(&self.buf(), self.cursor_line, self.cursor_col);
        let end = self.buf().line_col_to_offset(nl, nc).unwrap_or(start);
        if end > start {
            let content = self.buf().content();
            self.yank(content[start..end].to_string());
        }
    }

    pub(super) fn yank_to_end(&mut self) {
        let start = self
            .buf()
            .line_col_to_offset(self.cursor_line, self.cursor_col)
            .unwrap_or(0);
        let line_len = self.buf().line_len(self.cursor_line);
        let end = self
            .buf()
            .line_col_to_offset(self.cursor_line, line_len)
            .unwrap_or(start);
        if end > start {
            let content = self.buf().content();
            self.yank(content[start..end].to_string());
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
            "incsearch" | "is" => self.options.incsearch = true,
            "noincsearch" | "nois" => self.options.incsearch = false,
            "matchparen" => self.options.matchparen = true,
            "nomatchparen" => self.options.matchparen = false,
            "rainbow" => self.options.rainbow = true,
            "norainbow" => self.options.rainbow = false,
            "guides" => self.options.guides = true,
            "noguides" => self.options.guides = false,
            _ => {
                if let Some(style) = self.parse_cursor_set(opt) {
                    style
                } else {
                    self.status = format!("Unknown option: {opt}");
                }
            }
        }
    }

    fn parse_cursor_set(&mut self, opt: &str) -> Option<()> {
        let (key, val) = opt.split_once('=')?;
        let style = match val {
            "bar" => CursorStyle::Bar,
            "block" => CursorStyle::Block,
            "underline" => CursorStyle::Underline,
            "software" | "none" => CursorStyle::Software,
            _ => return None,
        };
        match key {
            "cursor_insert" => self.options.cursor_insert = style,
            "cursor_normal" => self.options.cursor_normal = style,
            "cursor_command" => self.options.cursor_command = style,
            _ => return None,
        }
        Some(())
    }

    pub(super) fn yank_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buf().line_count());
        let mut result = String::new();
        for i in self.cursor_line..end_line {
            result.push_str(&self.buf().line(i).unwrap_or_default());
            result.push('\n');
        }
        self.yank(result);
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
        let offset = self.buf().line_col_to_offset(self.cursor_line, 0).unwrap_or(0);
        self.buf().insert(offset, "\n");
        self.cursor_col = 0;
        self.mode = EditorMode::Insert;
    }

    pub(super) fn indent_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buf().line_count());
        for line in self.cursor_line..end_line {
            let offset = self.buf().line_col_to_offset(line, 0);
            if let Some(offset) = offset {
                self.buf().insert(offset, "    ");
            }
        }
    }

    pub(super) fn unindent_lines(&mut self, n: usize) {
        let end_line = (self.cursor_line + n).min(self.buf().line_count());
        for line in self.cursor_line..end_line {
            let text = self.buf().line(line).unwrap_or_default();
            let spaces = text.chars().take(4).take_while(|c| *c == ' ').count();
            if spaces > 0 {
                let offset = self.buf().line_col_to_offset(line, 0);
                if let Some(offset) = offset {
                    self.buf().delete(offset, offset + spaces);
                }
            }
        }
    }
}
