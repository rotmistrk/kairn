//! Visual mode methods.

use super::keymap::EditorMode;
use super::Editor;

impl Editor {
    pub(super) fn enter_visual(&mut self) {
        self.mode = EditorMode::Visual;
        self.visual_anchor = Some((self.cursor_line, self.cursor_col));
    }

    pub(super) fn enter_visual_line(&mut self) {
        self.mode = EditorMode::VisualLine;
        self.visual_anchor = Some((self.cursor_line, 0));
    }

    pub(super) fn exit_visual(&mut self) {
        self.mode = EditorMode::Normal;
        self.visual_anchor = None;
    }

    /// Get the visual selection range as (start_offset, end_offset).
    pub fn visual_range(&self) -> Option<(usize, usize)> {
        let (al, ac) = self.visual_anchor?;
        let (cl, cc) = (self.cursor_line, self.cursor_col);
        let anchor_off = self.buf().line_col_to_offset(al, ac)?;
        let cursor_off = self.buf().line_col_to_offset(cl, cc)?;
        let content = self.buf().content();
        let end_extra = content[cursor_off..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
        if anchor_off <= cursor_off {
            Some((anchor_off, cursor_off + end_extra))
        } else {
            let anchor_extra = content[anchor_off..].chars().next().map(|c| c.len_utf8()).unwrap_or(0);
            Some((cursor_off, anchor_off + anchor_extra))
        }
    }

    /// Get visual line range as (start_line, end_line).
    pub fn visual_line_range(&self) -> Option<(usize, usize)> {
        let (al, _) = self.visual_anchor?;
        let cl = self.cursor_line;
        Some((al.min(cl), al.max(cl)))
    }

    pub(super) fn visual_delete(&mut self) {
        if self.mode == EditorMode::VisualLine {
            if let Some((start_line, end_line)) = self.visual_line_range() {
                let start = self.buf().line_col_to_offset(start_line, 0).unwrap_or(0);
                let end = if end_line + 1 < self.buf().line_count() {
                    self.buf().line_col_to_offset(end_line + 1, 0).unwrap_or(start)
                } else {
                    self.buf().content().len()
                };
                if end > start {
                    let content = self.buf().content();
                    self.yank(content[start..end].to_string());
                    self.buf().delete(start, end);
                }
                let target = start_line.min(self.buf().line_count().saturating_sub(1));
                self.cursor_line = target;
                self.cursor_col = 0;
            }
        } else if let Some((start, end)) = self.visual_range() {
            if end > start {
                let content = self.buf().content();
                self.yank(content[start..end].to_string());
                self.buf().delete(start, end);
                let (l, c) = self.buf().offset_to_line_col(start);
                self.cursor_line = l;
                self.cursor_col = c;
            }
        }
        self.exit_visual();
    }

    pub(super) fn visual_yank(&mut self) {
        if self.mode == EditorMode::VisualLine {
            if let Some((start_line, end_line)) = self.visual_line_range() {
                let start = self.buf().line_col_to_offset(start_line, 0).unwrap_or(0);
                let end = if end_line + 1 < self.buf().line_count() {
                    self.buf().line_col_to_offset(end_line + 1, 0).unwrap_or(start)
                } else {
                    self.buf().content().len()
                };
                let content = self.buf().content();
                self.yank(content[start..end].to_string());
            }
        } else if let Some((start, end)) = self.visual_range() {
            let content = self.buf().content();
            self.yank(content[start..end].to_string());
        }
        self.exit_visual();
        self.status = "yanked".to_string();
    }

    pub(super) fn visual_change(&mut self) {
        self.buf().begin_group();
        self.visual_delete();
        self.mode = EditorMode::Insert;
    }

    pub(super) fn visual_ex_command(&mut self) {
        self.exit_visual();
        self.mode = EditorMode::Command;
        self.command_buf = "'<,'>".to_string();
    }

    pub(super) fn visual_indent(&mut self) {
        let (start_line, end_line) = match self.visual_line_range() {
            Some(r) => r,
            None => {
                let (al, _) = self.visual_anchor.unwrap_or((self.cursor_line, 0));
                (al.min(self.cursor_line), al.max(self.cursor_line))
            }
        };
        for line in (start_line..=end_line).rev() {
            let offset = self.buf().line_col_to_offset(line, 0);
            if let Some(offset) = offset {
                self.buf().insert(offset, "    ");
            }
        }
        self.exit_visual();
    }

    pub(super) fn visual_unindent(&mut self) {
        let (start_line, end_line) = match self.visual_line_range() {
            Some(r) => r,
            None => {
                let (al, _) = self.visual_anchor.unwrap_or((self.cursor_line, 0));
                (al.min(self.cursor_line), al.max(self.cursor_line))
            }
        };
        for line in (start_line..=end_line).rev() {
            let text = self.buf().line(line).unwrap_or_default();
            let remove = if text.starts_with("    ") {
                4
            } else if text.starts_with('\t') {
                1
            } else {
                text.chars().take_while(|c| c.is_whitespace()).count().min(4)
            };
            if remove > 0 {
                let start = self.buf().line_col_to_offset(line, 0).unwrap_or(0);
                let end = self.buf().line_col_to_offset(line, remove).unwrap_or(start);
                self.buf().delete(start, end);
            }
        }
        self.exit_visual();
    }
}
