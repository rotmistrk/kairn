//! Viewport scrolling (wrap-aware) and command history navigation.

use super::EditorView;

impl EditorView {
    pub(super) fn ensure_cursor_visible(&mut self) {
        let h = self.state.bounds().h as usize;
        if h == 0 {
            return;
        }
        self.editor.viewport_height = h;

        if self.editor.cursor_line < self.editor.viewport_scroll {
            self.editor.viewport_scroll = self.editor.cursor_line;
        }

        if !self.editor.options.wrap {
            if self.editor.cursor_line >= self.editor.viewport_scroll + h {
                self.editor.viewport_scroll = self.editor.cursor_line - h + 1;
            }
            // Horizontal scroll: keep cursor within visible columns
            let avail = self.text_avail_width();
            if avail > 0 {
                let col = self.cursor_visual_col();
                if col < self.editor.h_scroll {
                    self.editor.h_scroll = col;
                } else if col >= self.editor.h_scroll + avail {
                    self.editor.h_scroll = col - avail + 1;
                }
            }
            return;
        }

        // With wrap: count visual rows from viewport_scroll to cursor_line (inclusive)
        let avail = self.text_avail_width();
        if avail == 0 {
            return;
        }
        loop {
            let mut rows_used: usize = 0;
            for line_idx in self.editor.viewport_scroll..=self.editor.cursor_line {
                rows_used += self.wrapped_line_rows(line_idx, avail);
                if rows_used > h {
                    break;
                }
            }
            if rows_used <= h {
                break;
            }
            self.editor.viewport_scroll += 1;
        }
    }

    /// Number of visual rows a line occupies when wrapped.
    pub(super) fn wrapped_line_rows(&self, line_idx: usize, avail: usize) -> usize {
        let line = self.editor.buf().line(line_idx).unwrap_or_default();
        if line.is_empty() {
            return 1;
        }
        let tab_w = self.editor.options.tab_width;
        let mut col: usize = 0;
        let mut rows: usize = 1;
        for ch in line.chars() {
            let w = if ch == '\t' {
                tab_w
            } else {
                1
            };
            if col + w > avail && col > 0 {
                rows += 1;
                col = 0;
            }
            col += w;
        }
        rows
    }

    /// Available text width (total width minus gutter).
    pub(super) fn text_avail_width(&self) -> usize {
        let w = self.state.bounds().w;
        let gutter = self.gutter_width();
        w.saturating_sub(gutter) as usize
    }

    /// Visual column of cursor (accounts for tabs and wide chars).
    pub(super) fn cursor_visual_col(&self) -> usize {
        let line = self.editor.buf().line(self.editor.cursor_line).unwrap_or_default();
        let tab_w = self.editor.options.tab_width;
        let mut col: usize = 0;
        for (i, ch) in line.chars().enumerate() {
            if i == self.editor.cursor_col {
                return col;
            }
            col += if ch == '\t' {
                tab_w
            } else {
                txv_core::text::display_char_width(ch) as usize
            };
        }
        col
    }

    /// Compute visual (row, col) for cursor position accounting for wrapping.
    /// `line_start_row` is the first visual row of this buffer line.
    pub(super) fn cursor_visual_pos(
        &self,
        line_idx: usize,
        cursor_col: usize,
        avail: usize,
        tab_width: usize,
        line_start_row: usize,
    ) -> (usize, usize) {
        let line = self.editor.buf().line(line_idx).unwrap_or_default();
        let mut col: usize = 0;
        let mut vrow = line_start_row;
        for (i, ch) in line.chars().enumerate() {
            if i == cursor_col {
                return (vrow, col);
            }
            let w = if ch == '\t' {
                tab_width
            } else {
                1
            };
            if col + w > avail && col > 0 {
                vrow += 1;
                col = 0;
            }
            col += w;
        }
        // Cursor at or past end of line
        (vrow, col)
    }

    pub(super) fn history_prev(&mut self) {
        let hist = &self.editor.command_history;
        if hist.is_empty() {
            return;
        }
        let idx = match self.editor.history_index {
            None => {
                self.editor.history_prefix = self.editor.command_buf.clone();
                hist.len()
            }
            Some(i) => i,
        };
        let prefix = &self.editor.history_prefix;
        for i in (0..idx).rev() {
            if hist[i].starts_with(prefix.as_str()) {
                self.editor.history_index = Some(i);
                self.editor.command_buf = hist[i].clone();
                return;
            }
        }
    }

    pub(super) fn history_next(&mut self) {
        let Some(idx) = self.editor.history_index else {
            return;
        };
        let hist = &self.editor.command_history;
        let prefix = &self.editor.history_prefix;
        let found = hist
            .iter()
            .enumerate()
            .skip(idx + 1)
            .find(|(_, entry)| entry.starts_with(prefix.as_str()));
        if let Some((i, entry)) = found {
            self.editor.history_index = Some(i);
            self.editor.command_buf = entry.clone();
        } else {
            self.editor.history_index = None;
            self.editor.command_buf = self.editor.history_prefix.clone();
        }
    }
}
