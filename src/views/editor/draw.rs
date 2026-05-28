//! EditorView draw implementation.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use super::draw_style::{draw_indent_guides, rainbow_brackets};
use super::EditorView;
use crate::app_palette::app_palette;
use crate::editor::motions::match_bracket;
use crate::highlight::HlSpan;
use crate::views::editor::draw_diagnostics::diag_marker_style;

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
    pub(super) scroll: usize,
    pub(super) avail: usize,
    pub(super) wrap: bool,
    pub(super) h_off: usize,
    pub(super) tab_width: usize,
    pub(super) matchparen_pos: Option<(usize, usize)>,
    pub(super) matchparen_style: Style,
    pub(super) rainbow_map: Vec<(usize, Color)>,
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

        let mut row: usize = 0;
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

    fn build_draw_params(&self, w: u16, h: u16) -> DrawParams {
        let pal = palette();
        let app = app_palette();
        let gutter_w = self.gutter_width();
        let wrap = self.editor.options.wrap;
        let matchparen_pos = if self.editor.options.matchparen {
            match_bracket(&self.editor.buf(), self.editor.cursor_line, self.editor.cursor_col)
        } else {
            None
        };
        let rainbow_map = if self.editor.options.rainbow {
            rainbow_brackets(&self.editor.buf().line(self.editor.cursor_line).unwrap_or_default())
        } else {
            Vec::new()
        };
        DrawParams {
            w,
            h,
            gutter_w,
            gutter_style: app.editor().gutter(),
            cursor_style: app.editor().cursor(),
            hl_match_style: app.editor().highlight_match(),
            hl_other_bg: app.editor().highlight_other().bg,
            visual_bg: if self.state.is_focused() {
                pal.style(StyleId::VisualSelection).bg
            } else {
                pal.style(StyleId::CursorUnfocused).bg
            },
            scroll: self.editor.viewport_scroll,
            avail: w.saturating_sub(gutter_w) as usize,
            wrap,
            h_off: if wrap {
                0
            } else {
                self.editor.h_scroll
            },
            tab_width: self.editor.options.tab_width,
            matchparen_pos,
            matchparen_style: app.editor().matchparen(),
            rainbow_map,
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

        let visual_range = self.editor.visual_range();
        let highlight = self.editor.highlight.take();
        let (visual_row, col_offset) = self.draw_line_spans(
            spans,
            line_idx,
            p,
            (visual_range, highlight.as_ref()),
            (line_start_off, row),
        );
        self.editor.highlight = highlight;
        self.draw_line_tail(line_idx, &line, col_offset, visual_row, p, text_x);
        visual_row
    }

    fn draw_gutter(&mut self, line_idx: usize, y: u16, p: &DrawParams) {
        if p.gutter_w > 0 {
            let num = format!("{:>width$} ", line_idx + 1, width = (p.gutter_w - 1) as usize);
            self.state.buffer_mut().print(0, y, &num, p.gutter_style);
            if let Some(sev) = self.diagnostic_severity_at(line_idx) {
                let marker_style = diag_marker_style(sev);
                self.state.buffer_mut().put(p.gutter_w - 1, y, '●', marker_style);
            }
        }
    }

    fn draw_line_tail(
        &mut self,
        line_idx: usize,
        line: &str,
        mut col_offset: usize,
        visual_row: usize,
        p: &DrawParams,
        text_x: u16,
    ) {
        let normal = Style::default();
        let app = app_palette();
        if self.editor.options.list
            && col_offset >= p.h_off
            && col_offset - p.h_off < p.avail
            && visual_row < p.h as usize
        {
            let list_style = app.editor().list_chars();
            let x = text_x + (col_offset - p.h_off) as u16;
            self.state.buffer_mut().put(x, visual_row as u16, '$', list_style);
            col_offset += 1;
        }

        if visual_row < p.h as usize {
            let vy = visual_row as u16;
            for pad_col in col_offset..p.avail {
                self.state.buffer_mut().put(text_x + pad_col as u16, vy, ' ', normal);
            }
            if self.editor.options.guides {
                draw_indent_guides(
                    self.state.buffer_mut(),
                    line,
                    text_x,
                    vy,
                    p.tab_width,
                    p.avail,
                    p.gutter_style,
                );
            }
        }

        self.draw_line_cursor(line_idx, visual_row, p, text_x);
    }

    fn draw_line_cursor(&mut self, line_idx: usize, visual_row: usize, p: &DrawParams, text_x: u16) {
        if line_idx != self.editor.cursor_line || !self.state.is_focused() || self.uses_hw_cursor() {
            return;
        }
        let (cursor_vrow, cursor_vcol) = self.cursor_visual_pos(
            line_idx,
            self.editor.cursor_col,
            p.avail + p.h_off,
            p.tab_width,
            visual_row,
        );
        let screen_col = cursor_vcol.saturating_sub(p.h_off);
        if cursor_vrow < p.h as usize && cursor_vcol >= p.h_off && screen_col < p.avail {
            let cx = text_x + screen_col as u16;
            let cy = cursor_vrow as u16;
            let under = self.state.buffer_mut().cell(cx, cy).ch;
            self.state.buffer_mut().put(cx, cy, under, p.cursor_style);
        }
    }
}
