//! Diff drawing helpers: gutter, text rendering, utilities.

use txv_core::cell::Style;

use crate::app_palette::app_palette;
use crate::views::editor::diff_model::DiffState;
use crate::views::editor::EditorView;

use super::diff_model::DiffLine;

impl EditorView {
    pub(super) fn draw_diff_gutter(
        &mut self,
        x: u16,
        y: u16,
        dw: usize,
        base_line: Option<usize>,
        buf_line: Option<usize>,
    ) {
        if !self.editor.options.number {
            return;
        }
        let gs = app_palette().editor().gutter();
        let left = match base_line {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let right = match buf_line {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let gutter = format!("{} {} ", left, right);
        self.state.buffer_mut().print(x, y, &gutter, gs);
    }

    pub(super) fn draw_diff_text(&mut self, x: u16, y: u16, avail: usize, text: &str, style: Style) {
        use txv_core::text::display_char_width;
        let tab_w = self.editor.options.tab_width;
        let mut col = 0usize;
        for ch in text.chars() {
            if col >= avail {
                break;
            }
            if ch == '\t' {
                let spaces = tab_w - (col % tab_w);
                let end = (col + spaces).min(avail);
                while col < end {
                    self.state.buffer_mut().put(x + col as u16, y, ' ', style);
                    col += 1;
                }
            } else {
                self.state.buffer_mut().put(x + col as u16, y, ch, style);
                col += display_char_width(ch) as usize;
            }
        }
        while col < avail {
            self.state.buffer_mut().put(x + col as u16, y, ' ', style);
            col += 1;
        }
    }

    pub(super) fn max_base_line(&self, ds: &DiffState) -> usize {
        ds.lines
            .iter()
            .filter_map(|l| match l {
                DiffLine::Context { base_line, .. } | DiffLine::Deleted { base_line, .. } => Some(*base_line),
                _ => None,
            })
            .max()
            .unwrap_or(0)
            + 1
    }
}

pub(super) fn digit_width(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        (n as f64).log10() as usize + 1
    }
}
