//! DiffView side-by-side rendering — two-column layout with vertical divider.

use txv_core::cell::Style;
use txv_core::glyphs::glyphs;
use txv_core::palette::{palette, StyleId};

use crate::app_palette::app_palette;
use crate::views::editor::sbs_model::{split_for_side_by_side, SbsLine};

use super::sbs_cell::{draw_sbs_cell, SbsCellCtx};
use super::DiffView;

/// Layout and style info for SBS rendering.
pub(super) struct SbsLayout {
    pub(super) half_w: u16,
    pub(super) right_x: u16,
    pub(super) right_w: u16,
    pub(super) added: Style,
    pub(super) deleted: Style,
    pub(super) context: Style,
    pub(super) gap: Style,
    pub(super) divider: Style,
    pub(super) cursor: Style,
}

impl DiffView {
    pub(super) fn draw_sbs(&mut self) {
        let buf_text = self.buf_lines.join("\n");
        let (left, right) = split_for_side_by_side(&self.ds.lines, &self.base_text, &buf_text);
        let layout = self.build_sbs_layout();
        let h = self.content_height();
        let g = glyphs();
        for row in 0..h {
            let vi = self.ds.scroll + row;
            let y = row as u16;
            self.group
                .buffer_mut()
                .put(layout.half_w, y, g.ui().separator_v(), layout.divider);
            if vi >= left.len() {
                self.draw_sbs_empty_row(y, &layout);
            } else {
                self.draw_sbs_content_row(y, vi, &left, &right, &layout);
            }
        }
    }

    fn build_sbs_layout(&self) -> SbsLayout {
        let app = app_palette();
        let pal = palette();
        let w = self.width();
        let half_w = w.saturating_sub(1) / 2;
        SbsLayout {
            half_w,
            right_x: half_w + 1,
            right_w: w.saturating_sub(half_w + 1),
            added: app.diff().added(),
            deleted: app.diff().deleted(),
            context: Style::default(),
            gap: pal.style(StyleId::Dim),
            divider: pal.style(StyleId::Dim),
            cursor: if self.group.is_focused() {
                pal.style(StyleId::CursorFocused)
            } else {
                pal.style(StyleId::CursorUnfocused)
            },
        }
    }

    fn draw_sbs_empty_row(&mut self, y: u16, layout: &SbsLayout) {
        self.group
            .buffer_mut()
            .print_line(0, y, "~", layout.half_w, layout.context);
        self.group
            .buffer_mut()
            .print_line(layout.right_x, y, "~", layout.right_w, layout.context);
    }

    fn draw_sbs_content_row(&mut self, y: u16, vi: usize, left: &[SbsLine], right: &[SbsLine], layout: &SbsLayout) {
        let is_cursor = vi == self.ds.cursor;
        let lctx = self.make_cell_ctx(0, layout.half_w, y, is_cursor, true, layout);
        draw_sbs_cell(self.group.buffer_mut(), &left[vi], &lctx);
        let rl = right.get(vi).cloned().unwrap_or(SbsLine::Gap);
        let rctx = self.make_cell_ctx(layout.right_x, layout.right_w, y, is_cursor, false, layout);
        draw_sbs_cell(self.group.buffer_mut(), &rl, &rctx);
    }

    fn make_cell_ctx(&self, x: u16, w: u16, y: u16, is_cursor: bool, is_left: bool, l: &SbsLayout) -> SbsCellCtx {
        SbsCellCtx {
            x,
            w,
            y,
            is_cursor,
            is_left,
            added: l.added,
            deleted: l.deleted,
            context: l.context,
            gap: l.gap,
            cursor: l.cursor,
        }
    }
}
