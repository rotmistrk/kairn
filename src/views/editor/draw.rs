//! EditorView draw implementation.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use super::draw_style::{bracket_depth_at_line, draw_indent_guides, rainbow_brackets_with_depth};
use super::sticky_scroll;
use super::EditorView;
use crate::app_palette::app_palette;
use crate::editor::motions::match_bracket;
use crate::highlight::HlSpan;

/// Parameters collected for draw_editor rendering.
pub(super) struct DrawParams {
    pub(super) w: u16,
    pub(super) h: u16,
    pub(super) gutter_w: u16,
    pub(super) gutter_style: Style,
    pub(super) cursor_style: Style,
    pub(super) hl_match_style: Style,
    pub(super) hl_other_bg: Color,
    pub(super) visual_bg: Color,
    pub(super) ephemeral_bg: Color,
    pub(super) scroll: usize,
    pub(super) avail: usize,
    pub(super) wrap: bool,
    pub(super) h_off: usize,
    pub(super) tab_width: usize,
    pub(super) matchparen_pos: Option<(usize, usize)>,
    pub(super) matchparen_style: Style,
    /// Per-viewport-line rainbow maps: index is (line_idx - scroll).
    pub(super) rainbow_maps: Vec<Vec<(usize, Color)>>,
}

impl EditorView {
    pub(super) fn draw_editor(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        if self.in_diff_mode() {
            self.draw_diff();
            return;
        }
        if self.in_sbs_mode() {
            self.draw_sbs_diff();
            return;
        }

        let params = self.build_draw_params(w, h);
        let total_lines = self.editor.buf().line_count();
        let viewport_end = (params.scroll + h as usize).min(total_lines);
        let viewport_spans = self.compute_viewport_spans(params.scroll, viewport_end);

        let sticky_lines = sticky_scroll::compute_sticky_lines(&self.editor, params.scroll);
        let sticky_h = sticky_lines.len() as u16;

        // Draw sticky headers at top
        for (i, sl) in sticky_lines.iter().enumerate() {
            sticky_scroll::draw_sticky(self.state.buffer_mut(), sl, i as u16, w);
        }

        let mut row: usize = sticky_h as usize;
        let mut line_idx = params.scroll;

        while row < h as usize && line_idx < viewport_end {
            let visual_row = self.draw_editor_line(line_idx, row, &params, &viewport_spans);
            row = visual_row + 1;
            line_idx += 1;
        }

        if let Some((hl_line, col_start, col_end)) = self.highlight_word {
            self.paint_highlight_word(hl_line, col_start, col_end, params.scroll);
        }
        self.draw_footer(row, params.gutter_style);
    }

    fn compute_rainbow_maps(&self, scroll: usize, viewport_end: usize) -> Vec<Vec<(usize, Color)>> {
        if !self.editor.options().rainbow() {
            return Vec::new();
        }
        let mut depth = bracket_depth_at_line(&self.editor.buf(), scroll);
        let mut maps = Vec::with_capacity(viewport_end - scroll);
        for i in scroll..viewport_end {
            let line = self.editor.buf().line(i).unwrap_or_default();
            let (map, new_depth) = rainbow_brackets_with_depth(&line, depth);
            maps.push(map);
            depth = new_depth;
        }
        maps
    }

    fn build_draw_params(&self, w: u16, h: u16) -> DrawParams {
        let pal = palette();
        let app = app_palette();
        let gutter_w = self.gutter_width();
        let wrap = self.editor.options().wrap();
        let matchparen_pos = if self.editor.options().matchparen() {
            match_bracket(&self.editor.buf(), self.editor.cursor_line(), self.editor.cursor_col())
        } else {
            None
        };
        let scroll = self.editor.viewport_scroll();
        let total_lines = self.editor.buf().line_count();
        let viewport_end = (scroll + h as usize).min(total_lines);
        DrawParams {
            w,
            h,
            gutter_w,
            gutter_style: app.editor().gutter(),
            cursor_style: app.editor().cursor(),
            hl_match_style: app.editor().highlight_match(),
            hl_other_bg: app.editor().highlight_other().bg(),
            visual_bg: if self.state.is_focused() {
                pal.style(StyleId::VisualSelection).bg()
            } else {
                pal.style(StyleId::CursorUnfocused).bg()
            },
            ephemeral_bg: app.editor().highlight_other().bg(),
            scroll,
            avail: w.saturating_sub(gutter_w) as usize,
            wrap,
            h_off: if wrap {
                0
            } else {
                self.editor.h_scroll()
            },
            tab_width: self.editor.options().tab_width(),
            matchparen_pos,
            matchparen_style: app.editor().matchparen(),
            rainbow_maps: self.compute_rainbow_maps(scroll, viewport_end),
        }
    }

    fn draw_editor_line(
        &mut self,
        line_idx: usize,
        row: usize,
        p: &DrawParams,
        viewport_spans: &[Vec<HlSpan>],
    ) -> usize {
        let text_x = p.gutter_w;
        self.draw_gutter(line_idx, row as u16, p);

        let line = self.editor.buf().line(line_idx).unwrap_or_default();
        let line_start_off = self.editor.buf().line_col_to_offset(line_idx, 0).unwrap_or(0);
        let default_spans;
        let spans: &[HlSpan] = match viewport_spans.get(line_idx - p.scroll) {
            Some(s) => s,
            None => {
                default_spans = [HlSpan::plain(line.clone())];
                &default_spans
            }
        };

        let visual_range = self.block_visual_range_for_line(line_idx);
        let highlight = self.editor.take_highlight();
        let (visual_row, col_offset) = self.draw_line_spans(
            spans,
            line_idx,
            p,
            (visual_range, highlight.as_ref()),
            (line_start_off, row),
        );
        self.editor.set_highlight(highlight);
        let fill = self.ephemeral_fill(line_idx, p);
        self.draw_line_tail(&line, col_offset, (visual_row, row), p, text_x, fill);
        self.draw_line_cursor(line_idx, row, p, text_x);
        visual_row
    }

    /// Compute visual byte range for a line, handling block mode per-line.
    fn block_visual_range_for_line(&self, line_idx: usize) -> Option<(usize, usize)> {
        use crate::editor::keymap::EditorMode;
        if self.editor.mode() == EditorMode::VisualBlock {
            let (sl, el, sc, ec) = self.editor.block_range()?;
            if line_idx < sl || line_idx > el {
                return None;
            }
            let start = self.editor.buf().line_col_to_offset(line_idx, sc)?;
            let end = self
                .editor
                .buf()
                .line_col_to_offset(line_idx, ec + 1)
                .unwrap_or_else(|| {
                    // Line is shorter than ec+1: extend to end of line content
                    let line_start = self.editor.buf().line_col_to_offset(line_idx, 0).unwrap_or(start);
                    let line = self.editor.buf().line(line_idx).unwrap_or_default();
                    line_start + line.len()
                });
            Some((start, end))
        } else {
            self.editor.visual_range()
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_line_tail(
        &mut self,
        line: &str,
        mut col_offset: usize,
        rows: (usize, usize), // (visual_row, start_row)
        p: &DrawParams,
        text_x: u16,
        fill: Style,
    ) {
        let (visual_row, start_row) = rows;
        if self.editor.options().list()
            && col_offset >= p.h_off
            && (col_offset - p.h_off) < p.avail
            && visual_row < p.h as usize
        {
            let x = text_x + (col_offset - p.h_off) as u16;
            self.state
                .buffer_mut()
                .put(x, visual_row as u16, '$', app_palette().editor().list_chars());
            col_offset += 1;
        }
        if visual_row >= p.h as usize {
            return;
        }
        let vy = visual_row as u16;
        for pad_col in col_offset..p.avail {
            self.state.buffer_mut().put(text_x + pad_col as u16, vy, ' ', fill);
        }
        if self.editor.options().guides() {
            draw_indent_guides(
                self.state.buffer_mut(),
                line,
                text_x,
                start_row as u16,
                p.tab_width,
                p.avail,
                p.gutter_style,
            );
        }
    }

    fn draw_line_cursor(&mut self, line_idx: usize, visual_row: usize, p: &DrawParams, text_x: u16) {
        if line_idx != self.editor.cursor_line() || !self.state.is_focused() || self.uses_hw_cursor() {
            return;
        }
        let (cursor_vrow, cursor_vcol) = self.cursor_visual_pos(
            line_idx,
            self.editor.cursor_col(),
            p.avail + p.h_off,
            p.tab_width,
            visual_row,
        );
        let screen_col = cursor_vcol.saturating_sub(p.h_off);
        if cursor_vrow < p.h as usize && cursor_vcol >= p.h_off && screen_col < p.avail {
            let cx = text_x + screen_col as u16;
            let cy = cursor_vrow as u16;
            let under = self.state.buffer_mut().cell(cx, cy).ch();
            self.state.buffer_mut().put(cx, cy, under, p.cursor_style);
        }
    }
}
