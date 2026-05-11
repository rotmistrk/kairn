//! PtyTerminal scrollback rendering and content extraction.

use txv_core::prelude::*;

use crate::pty_terminal::PtyTerminal;

impl PtyTerminal {
    /// Return the last `max_lines` lines from scrollback + visible grid as strings.
    pub fn get_content(&self, max_lines: usize) -> Vec<String> {
        let grid_rows = self.termbuf.grid_rows() as usize;
        let sb_len = self.termbuf.scrollback_len();
        let total = sb_len + grid_rows;
        let take = max_lines.min(total);
        let mut result = Vec::with_capacity(take);
        let start = total.saturating_sub(take);
        for i in start..total {
            let line_str = if i < sb_len {
                if let Some(line) = self.termbuf.scrollback_line(sb_len - 1 - i) {
                    line.iter().map(|c| c.ch).collect::<String>().trim_end().to_string()
                } else {
                    String::new()
                }
            } else {
                let row = i - sb_len;
                if let Some(line) = self.termbuf.grid_line(row) {
                    line.iter().map(|c| c.ch).collect::<String>().trim_end().to_string()
                } else {
                    String::new()
                }
            };
            result.push(line_str);
        }
        result
    }

    pub(crate) fn draw_scrollback(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        let h = b.h as usize;
        let w = b.w as usize;
        let grid_rows = self.termbuf.grid_rows() as usize;
        let sb_len = self.termbuf.scrollback_len();
        let style = txv_core::cell::Style::default();
        let total = sb_len + grid_rows;
        let bottom_line = total.saturating_sub(self.scroll_offset);
        let top_line = bottom_line.saturating_sub(h);
        for screen_y in 0..h {
            let line_idx = top_line + screen_y;
            if line_idx < sb_len {
                if let Some(line) = self.termbuf.scrollback_line(sb_len - 1 - line_idx) {
                    for (x, tc) in line.iter().enumerate().take(w) {
                        surface.put(b.x + x as u16, b.y + screen_y as u16, tc.ch, tc.style);
                    }
                }
            } else {
                let row = line_idx - sb_len;
                if let Some(line) = self.termbuf.grid_line(row) {
                    for (x, tc) in line.iter().enumerate().take(w) {
                        surface.put(b.x + x as u16, b.y + screen_y as u16, tc.ch, tc.style);
                    }
                } else {
                    for x in 0..w {
                        surface.put(b.x + x as u16, b.y + screen_y as u16, ' ', style);
                    }
                }
            }
        }
    }
}
