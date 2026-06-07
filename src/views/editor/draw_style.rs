//! Style helpers for editor draw — rainbow brackets and highlight word.

use txv_core::prelude::Color;

// Re-export from txv-edit (no duplication)
pub(super) use txv_edit::view::draw::rainbow::rainbow_brackets_with_depth;

pub(super) fn rainbow_brackets(line: &str) -> Vec<(usize, Color)> {
    rainbow_brackets_with_depth(line, 0).0
}

impl super::EditorView {
    /// Paint highlight on a word (gs target), accounting for wrapped lines.
    pub(super) fn paint_highlight_word(&mut self, hl_line: usize, col_start: usize, col_end: usize, scroll: usize) {
        if hl_line < scroll {
            return;
        }
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        let gutter_w = self.gutter_width();
        let avail = w.saturating_sub(gutter_w) as usize;
        let mut vis_row: usize = 0;
        for li in scroll..hl_line {
            vis_row += self.wrapped_line_rows(li, avail);
        }
        if vis_row >= h as usize {
            return;
        }
        let app = crate::app_palette::app_palette();
        let bg = app.editor().highlight_match().bg();
        let x_start = gutter_w + col_start as u16;
        let x_end = gutter_w + (col_end as u16).min(w.saturating_sub(gutter_w));
        let y = vis_row as u16;
        for x in x_start..x_end {
            self.state.buffer_mut().cell_mut(x, y).style_mut().set_bg(bg);
        }
    }
}

#[cfg(test)]
#[path = "draw_style_tests.rs"]
mod tests;
