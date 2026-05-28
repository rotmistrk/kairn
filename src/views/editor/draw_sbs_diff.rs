//! Side-by-side diff rendering — both columns in a single view.

use txv_core::glyphs::glyphs;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use super::sbs_model::SbsLine;
use super::EditorView;
use crate::app_palette::app_palette;
use crate::editor::keymap::EditorMode;

impl EditorView {
    pub(super) fn draw_sbs_diff(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        let sbs = match self.sbs_state.take() {
            Some(s) => s,
            None => return,
        };

        let app = app_palette();
        let pal = palette();
        let styles = SbsStyles {
            added: app.diff().added(),
            deleted: app.diff().deleted(),
            context: Style::default(),
            fold: app.diff().fold(),
            gap: pal.style(StyleId::Dim),
            divider: pal.style(StyleId::Dim),
            gutter: app.editor().gutter(),
            cursor: if self.state.is_focused() {
                pal.style(StyleId::CursorFocused)
            } else {
                pal.style(StyleId::CursorUnfocused)
            },
        };

        let (dw, gutter_w) = self.sbs_gutter_dims(&sbs);
        let half_w = w.saturating_sub(1) / 2;
        let g = glyphs();

        self.draw_sbs_rows(&sbs, (w, h, half_w), (gutter_w, dw), &g, &styles);
        self.draw_sbs_prompt(w, h);
        self.sbs_state = Some(sbs);
    }

    fn sbs_gutter_dims(&self, sbs: &super::sbs_model::SbsDiffState) -> (usize, u16) {
        let show_numbers = self.editor.options.number;
        if !show_numbers {
            return (0, 0);
        }
        let max_line = sbs
            .left
            .iter()
            .chain(sbs.right.iter())
            .filter_map(|l| match l {
                SbsLine::Content { line_no, .. } => Some(*line_no),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        let dw = digit_width(max_line);
        (dw, (dw + 1) as u16)
    }

    #[allow(clippy::too_many_arguments)]
    #[rustfmt::skip]
    fn draw_sbs_rows(
        &mut self, sbs: &super::sbs_model::SbsDiffState,
        dims: (u16, u16, u16), gutter: (u16, usize),
        g: &txv_core::glyphs::GlyphSet, styles: &SbsStyles,
    ) {
        let (w, h, half_w) = dims;
        let right_x = half_w + 1;
        let right_w = w.saturating_sub(right_x);
        for row in 0..h as usize {
            let vi = sbs.scroll + row;
            let y = row as u16;
            self.state.buffer_mut().put(half_w, y, g.ui.separator_v, styles.divider);
            if vi >= sbs.left.len() {
                self.state.buffer_mut().print_line(0, y, "~", half_w, styles.context);
                self.state.buffer_mut().print_line(right_x, y, "~", right_w, styles.context);
                continue;
            }
            let is_cursor = vi == sbs.cursor;
            let buf = self.state.buffer_mut();
            draw_sbs_line(buf, &sbs.left[vi], (0, half_w, y), is_cursor, true, gutter, styles);
            let buf = self.state.buffer_mut();
            let right_line = sbs.right.get(vi).unwrap_or(&SbsLine::Gap);
            draw_sbs_line(buf, right_line, (right_x, right_w, y), is_cursor, false, gutter, styles);
        }
    }

    fn draw_sbs_prompt(&mut self, w: u16, h: u16) {
        if self.editor.mode == EditorMode::Command || self.editor.mode == EditorMode::Search {
            let prompt_y = h.saturating_sub(1);
            let prompt_style = palette().style(StyleId::StatusBar);
            let prefix = if self.editor.mode == EditorMode::Search {
                "/"
            } else {
                ":"
            };
            let prompt_text = format!("{}{}", prefix, self.editor.command_buf);
            self.state
                .buffer_mut()
                .print_line(0, prompt_y, &prompt_text, w, prompt_style);
        }
    }
}

struct SbsStyles {
    added: Style,
    deleted: Style,
    context: Style,
    fold: Style,
    gap: Style,
    divider: Style,
    gutter: Style,
    cursor: Style,
}

#[allow(clippy::too_many_arguments)]
fn draw_sbs_line(
    buf: &mut Buffer,
    line: &SbsLine,
    pos: (u16, u16, u16),
    is_cursor: bool,
    is_left: bool,
    gutter: (u16, usize),
    styles: &SbsStyles,
) {
    let (x, w, y) = pos;
    let (gutter_w, dw) = gutter;
    let text_x = x + gutter_w;
    let text_w = w.saturating_sub(gutter_w);

    match line {
        SbsLine::Content { line_no, text, changed } => {
            let st = sbs_content_style(is_cursor, *changed, is_left, styles);
            if gutter_w > 0 {
                let gutter = format!("{:>width$} ", line_no + 1, width = dw);
                buf.print(x, y, &gutter, styles.gutter);
            }
            buf.print_line(text_x, y, text, text_w, st);
        }
        SbsLine::Gap => {
            let st = if is_cursor {
                styles.cursor
            } else {
                styles.gap
            };
            buf.print_line(x, y, "", w, st);
        }
        SbsLine::Folded { count } => {
            let st = if is_cursor {
                styles.cursor
            } else {
                styles.fold
            };
            let label = format!("--- {} lines ---", count);
            buf.print_line(x, y, &label, w, st);
        }
    }
}

fn sbs_content_style(is_cursor: bool, changed: bool, is_left: bool, styles: &SbsStyles) -> Style {
    if is_cursor {
        styles.cursor
    } else if changed {
        if is_left {
            styles.deleted
        } else {
            styles.added
        }
    } else {
        styles.context
    }
}

fn digit_width(max: usize) -> usize {
    if max < 10 {
        1
    } else if max < 100 {
        2
    } else if max < 1000 {
        3
    } else if max < 10000 {
        4
    } else {
        5
    }
}
