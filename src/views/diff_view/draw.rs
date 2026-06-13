//! DiffView rendering — unified diff with dual gutter.

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;
use txv_core::text::display_char_width;

use crate::app_palette::app_palette;
use crate::views::editor::diff_model::DiffLine;

use super::DiffView;

struct DrawCtx {
    w: u16,
    dw: usize,
    gutter_w: u16,
    text_x: u16,
    avail: usize,
    gutter_style: Style,
    added_style: Style,
    deleted_style: Style,
    context_style: Style,
    fold_style: Style,
    cursor_style: Style,
}

impl DiffView {
    pub(super) fn draw_unified(&mut self) {
        let dc = self.build_draw_ctx();
        let draw_h = self.content_height();

        for row in 0..draw_h {
            let vi = self.ds.scroll + row;
            let y = row as u16;
            let is_cursor = vi == self.ds.cursor;

            if vi >= self.ds.lines.len() {
                self.group.buffer_mut().print_line(0, y, "~", dc.w, dc.context_style);
                continue;
            }
            let line = self.ds.lines[vi].clone();
            self.draw_line(&dc, y, &line, is_cursor);
        }
    }

    fn build_draw_ctx(&self) -> DrawCtx {
        let app = app_palette();
        let pal = palette();
        let max_base = self.max_base_line();
        let max_buf = self.buf_lines.len();
        let dw = digit_width(max_base.max(max_buf));
        let w = self.width();
        let gutter_w = if self.show_numbers {
            (dw * 2 + 2) as u16
        } else {
            0
        };
        DrawCtx {
            w,
            dw,
            gutter_w,
            text_x: gutter_w,
            avail: w.saturating_sub(gutter_w) as usize,
            gutter_style: app.editor().gutter(),
            added_style: app.diff().added(),
            deleted_style: app.diff().deleted(),
            context_style: Style::default(),
            fold_style: app.diff().fold(),
            cursor_style: if self.group.is_focused() {
                pal.style(StyleId::CursorFocused)
            } else {
                pal.style(StyleId::CursorUnfocused)
            },
        }
    }

    fn draw_line(&mut self, dc: &DrawCtx, y: u16, line: &DiffLine, is_cursor: bool) {
        match line {
            DiffLine::Context { buf_line, base_line } => {
                self.draw_gutter(0, y, dc.dw, Some(*base_line), Some(*buf_line), dc.gutter_style);
                let text = self.buf_lines.get(*buf_line).map(|s| s.as_str()).unwrap_or("");
                let st = if is_cursor {
                    dc.cursor_style
                } else {
                    dc.context_style
                };
                draw_text(self.group.buffer_mut(), dc.text_x, y, dc.avail, text, st);
            }
            DiffLine::Added { buf_line } => {
                self.draw_gutter(0, y, dc.dw, None, Some(*buf_line), dc.gutter_style);
                let text = self.buf_lines.get(*buf_line).map(|s| s.as_str()).unwrap_or("");
                let st = if is_cursor {
                    dc.cursor_style
                } else {
                    dc.added_style
                };
                draw_text(self.group.buffer_mut(), dc.text_x, y, dc.avail, text, st);
            }
            DiffLine::Deleted { text, base_line } => {
                self.draw_gutter(0, y, dc.dw, Some(*base_line), None, dc.gutter_style);
                let st = if is_cursor {
                    dc.cursor_style
                } else {
                    dc.deleted_style
                };
                draw_text(self.group.buffer_mut(), dc.text_x, y, dc.avail, text, st);
            }
            DiffLine::Folded { count } => {
                let label = format!("--- {count} lines ---");
                let st = if is_cursor {
                    dc.cursor_style
                } else {
                    dc.fold_style
                };
                self.group.buffer_mut().print_line(0, y, &label, dc.w, st);
            }
        }
    }

    fn draw_gutter(&mut self, x: u16, y: u16, dw: usize, base: Option<usize>, buf: Option<usize>, style: Style) {
        if !self.show_numbers {
            return;
        }
        let left = match base {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let right = match buf {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let gutter = format!("{left} {right} ");
        self.group.buffer_mut().print(x, y, &gutter, style);
    }

    fn max_base_line(&self) -> usize {
        self.ds
            .lines
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

fn draw_text(buf: &mut Buffer, x: u16, y: u16, avail: usize, text: &str, style: Style) {
    let mut col = 0usize;
    for ch in text.chars() {
        if col >= avail {
            break;
        }
        if ch == '\t' {
            let spaces = 4 - (col % 4);
            let end = (col + spaces).min(avail);
            while col < end {
                buf.put(x + col as u16, y, ' ', style);
                col += 1;
            }
        } else {
            buf.put(x + col as u16, y, ch, style);
            col += display_char_width(ch) as usize;
        }
    }
    while col < avail {
        buf.put(x + col as u16, y, ' ', style);
        col += 1;
    }
}

fn digit_width(n: usize) -> usize {
    if n < 10 {
        1
    } else if n < 100 {
        2
    } else if n < 1000 {
        3
    } else if n < 10000 {
        4
    } else {
        5
    }
}
