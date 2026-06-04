//! Visual block mode operations.

use std::iter::repeat_n;

use super::keymap::EditorMode;
use super::Editor;

impl Editor {
    /// Delete the block selection. Pads short lines to maintain alignment.
    pub(super) fn block_delete(&mut self) {
        let Some((sl, el, sc, ec)) = self.block_range() else {
            self.exit_visual();
            return;
        };
        let mut yanked_lines: Vec<String> = Vec::new();
        self.buf().begin_group();
        // Process lines in reverse to keep offsets stable
        for line_idx in (sl..=el).rev() {
            let line = self.buf().line(line_idx).unwrap_or_default();
            let chars: Vec<char> = line.chars().collect();
            let line_len = chars.len();
            // Extract the block slice for yank
            let slice: String = chars[sc.min(line_len)..ec.min(line_len).saturating_add(1).min(line_len)]
                .iter()
                .collect();
            yanked_lines.push(slice);
            if sc >= line_len {
                continue; // line too short, nothing to delete
            }
            let del_end = (ec + 1).min(line_len);
            let start_off = self.buf().line_col_to_offset(line_idx, sc).unwrap_or(0);
            let end_off = self.buf().line_col_to_offset(line_idx, del_end).unwrap_or(start_off);
            if end_off > start_off {
                self.buf().delete(start_off, end_off);
            }
        }
        self.buf().end_group();
        yanked_lines.reverse();
        self.yank_block(yanked_lines.join("\n"));
        self.cursor_line = sl;
        self.cursor_col = sc;
        self.exit_visual();
    }

    /// Yank the block selection.
    pub(super) fn block_yank(&mut self) {
        let Some((sl, el, sc, ec)) = self.block_range() else {
            self.exit_visual();
            return;
        };
        let mut lines: Vec<String> = Vec::new();
        for line_idx in sl..=el {
            let line = self.buf().line(line_idx).unwrap_or_default();
            let chars: Vec<char> = line.chars().collect();
            let line_len = chars.len();
            let slice: String = chars[sc.min(line_len)..ec.min(line_len).saturating_add(1).min(line_len)]
                .iter()
                .collect();
            lines.push(slice);
        }
        self.yank_block(lines.join("\n"));
        self.cursor_line = sl;
        self.cursor_col = sc;
        self.exit_visual();
        self.status = "block yanked".to_string();
    }

    /// Paste block before cursor column. Pads short lines with spaces.
    pub(super) fn block_paste_before(&mut self) {
        let reg = self.register();
        if !self.register_block() || reg.is_empty() {
            return;
        }
        let block_lines: Vec<&str> = reg.lines().collect();
        let col = self.cursor_col;
        let start_line = self.cursor_line;
        self.buf().begin_group();
        for (i, block_line) in block_lines.iter().enumerate() {
            let line_idx = start_line + i;
            self.ensure_line_exists(line_idx);
            self.pad_line_to_col(line_idx, col);
            let off = self.buf().line_col_to_offset(line_idx, col).unwrap_or(0);
            self.buf().insert(off, block_line);
        }
        self.buf().end_group();
    }

    /// Paste block after cursor column. Pads short lines with spaces.
    pub(super) fn block_paste_after(&mut self) {
        let reg = self.register();
        if !self.register_block() || reg.is_empty() {
            return;
        }
        let block_lines: Vec<&str> = reg.lines().collect();
        let col = self.cursor_col + 1;
        let start_line = self.cursor_line;
        self.buf().begin_group();
        for (i, block_line) in block_lines.iter().enumerate() {
            let line_idx = start_line + i;
            self.ensure_line_exists(line_idx);
            self.pad_line_to_col(line_idx, col);
            let off = self.buf().line_col_to_offset(line_idx, col).unwrap_or(0);
            self.buf().insert(off, block_line);
        }
        self.buf().end_group();
    }

    /// Block change: delete block, enter insert mode (single-line; multi-line on exit).
    pub(super) fn block_change(&mut self) {
        self.buf().begin_group();
        self.block_delete();
        self.mode = EditorMode::Insert;
    }

    /// Block replace: replace every char in block with given char.
    pub(super) fn block_replace(&mut self, ch: char) {
        let Some((sl, el, sc, ec)) = self.block_range() else {
            self.exit_visual();
            return;
        };
        self.buf().begin_group();
        for line_idx in sl..=el {
            let line = self.buf().line(line_idx).unwrap_or_default();
            let chars: Vec<char> = line.chars().collect();
            let line_len = chars.len();
            if sc >= line_len {
                continue;
            }
            let replace_end = (ec + 1).min(line_len);
            let start_off = self.buf().line_col_to_offset(line_idx, sc).unwrap_or(0);
            let end_off = self
                .buf()
                .line_col_to_offset(line_idx, replace_end)
                .unwrap_or(start_off);
            if end_off > start_off {
                let replacement: String = repeat_n(ch, replace_end - sc).collect();
                self.buf().delete(start_off, end_off);
                self.buf().insert(start_off, &replacement);
            }
        }
        self.buf().end_group();
        self.exit_visual();
    }

    /// Ensure line exists (append newlines if needed).
    fn ensure_line_exists(&mut self, line_idx: usize) {
        let count = self.buf().line_count();
        if line_idx >= count {
            let end = self.buf().content().len();
            let newlines: String = "\n".repeat(line_idx - count + 1);
            self.buf().insert(end, &newlines);
        }
    }

    /// Pad a line with spaces so it has at least `col` characters.
    fn pad_line_to_col(&mut self, line_idx: usize, col: usize) {
        let line = self.buf().line(line_idx).unwrap_or_default();
        let line_len = line.chars().count();
        if line_len < col {
            let padding: String = " ".repeat(col - line_len);
            let end_off = self.buf().line_col_to_offset(line_idx, line_len).unwrap_or(0);
            self.buf().insert(end_off, &padding);
        }
    }
}
