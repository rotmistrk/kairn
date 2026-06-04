//! Character-level drawing helpers for the editor view.

use txv_core::prelude::*;

use super::draw::DrawParams;
use super::draw_style::{bracket_highlight, char_style};
use super::EditorView;
use crate::app_palette::{app_palette, AppPalette};
use crate::editor::highlight_state::HighlightState;
use crate::highlight::HlSpan;

/// Mutable drawing state passed through character rendering.
pub(super) struct CharDrawState {
    pub(super) col_offset: usize,
    pub(super) char_idx: usize,
    pub(super) byte_pos: usize,
    pub(super) visual_row: usize,
}

/// Visual highlight context: (visual_range, highlight ref).
type HlCtx<'a> = (Option<(usize, usize)>, Option<&'a HighlightState>);

impl EditorView {
    /// Draw all spans for a line. Returns (visual_row, col_offset).
    /// `init` is (line_start_off, row).
    pub(super) fn draw_line_spans(
        &mut self,
        spans: &[HlSpan],
        line_idx: usize,
        p: &DrawParams,
        hl: HlCtx,
        init: (usize, usize),
    ) -> (usize, usize) {
        let mut st = CharDrawState {
            col_offset: 0,
            char_idx: 0,
            byte_pos: init.0,
            visual_row: init.1,
        };
        for span in spans {
            if !self.draw_span_chars(span, line_idx, p, &hl, &mut st) {
                break;
            }
        }
        (st.visual_row, st.col_offset)
    }

    fn draw_span_chars(
        &mut self,
        span: &HlSpan,
        line_idx: usize,
        p: &DrawParams,
        hl: &HlCtx,
        st: &mut CharDrawState,
    ) -> bool {
        let app = app_palette();
        for ch in span.text.chars() {
            if ch == '\t' {
                self.draw_tab_char(span.style, p, hl, &app, st);
                st.char_idx += 1;
                st.byte_pos += ch.len_utf8();
                continue;
            }
            if !self.process_non_tab(ch, span.style, line_idx, p, hl, st) {
                return false;
            }
        }
        st.visual_row < p.h as usize
    }

    fn process_non_tab(
        &mut self,
        ch: char,
        span_style: Style,
        line_idx: usize,
        p: &DrawParams,
        hl: &HlCtx,
        st: &mut CharDrawState,
    ) -> bool {
        if !self.advance_char_position(ch, p, st) {
            return true;
        }
        if st.visual_row >= p.h as usize {
            return false;
        }
        if st.col_offset < p.h_off {
            st.col_offset += display_char_width(ch) as usize;
            st.char_idx += 1;
            st.byte_pos += ch.len_utf8();
            return true;
        }
        self.put_visible_char(ch, span_style, line_idx, p, hl, st);
        true
    }

    fn draw_tab_char(
        &mut self,
        span_style: Style,
        p: &DrawParams,
        hl: &HlCtx,
        app: &AppPalette,
        st: &mut CharDrawState,
    ) {
        let text_x = p.gutter_w;
        let char_st = char_style(
            span_style,
            st.byte_pos,
            hl.0,
            p.visual_bg,
            hl.1,
            p.hl_match_style,
            p.hl_other_bg,
        );
        for ti in 0..p.tab_width {
            if st.col_offset >= p.h_off + p.avail || st.visual_row >= p.h as usize {
                break;
            }
            if st.col_offset >= p.h_off {
                self.put_tab_cell(ti, p, text_x, app, char_st, st);
            }
            st.col_offset += 1;
        }
    }

    fn put_tab_cell(
        &mut self,
        ti: usize,
        p: &DrawParams,
        text_x: u16,
        app: &AppPalette,
        st: Style,
        draw_st: &CharDrawState,
    ) {
        let x = text_x + (draw_st.col_offset - p.h_off) as u16;
        let vy = draw_st.visual_row as u16;
        if self.editor.options.list {
            let ls = Style {
                fg: app.editor().list_chars().fg,
                ..st
            };
            let c = if ti == p.tab_width - 1 {
                '\u{2192}'
            } else {
                '\u{2500}'
            };
            self.state.buffer_mut().put(x, vy, c, ls);
        } else {
            self.state.buffer_mut().put(x, vy, ' ', st);
        }
    }

    fn advance_char_position(&mut self, ch: char, p: &DrawParams, st: &mut CharDrawState) -> bool {
        let text_x = p.gutter_w;
        if p.wrap {
            if st.col_offset >= p.avail {
                let vy = st.visual_row as u16;
                let normal = Style::default();
                for pad_col in st.col_offset..p.avail {
                    self.state.buffer_mut().put(text_x + pad_col as u16, vy, ' ', normal);
                }
                st.col_offset = 0;
                st.visual_row += 1;
                if st.visual_row >= p.h as usize {
                    return true;
                }
                if p.gutter_w > 0 {
                    let wy = st.visual_row as u16;
                    self.state
                        .buffer_mut()
                        .print_line(0, wy, "", p.gutter_w, p.gutter_style);
                }
            }
        } else if st.col_offset >= p.h_off + p.avail {
            st.char_idx += 1;
            st.byte_pos += ch.len_utf8();
            return false;
        }
        true
    }

    fn put_visible_char(
        &mut self,
        ch: char,
        span_style: Style,
        line_idx: usize,
        p: &DrawParams,
        hl: &HlCtx,
        st: &mut CharDrawState,
    ) {
        let text_x = p.gutter_w;
        let x = text_x + (st.col_offset - p.h_off) as u16;
        let style = char_style(
            span_style,
            st.byte_pos,
            hl.0,
            p.visual_bg,
            hl.1,
            p.hl_match_style,
            p.hl_other_bg,
        );
        let app = app_palette();
        let (display_ch, display_style) = self.resolve_display_char(ch, style, &app);
        let vy = st.visual_row as u16;
        let rainbow_map = self.rainbow_map_for_line(line_idx, p);
        let display_style = bracket_highlight(
            display_style,
            line_idx,
            st.char_idx,
            p.matchparen_pos,
            &p.matchparen_style,
            rainbow_map,
        );
        let display_style = self.apply_ephemeral_bg(display_style, line_idx, p);
        self.state.buffer_mut().put(x, vy, display_ch, display_style);
        st.col_offset += display_char_width(ch) as usize;
        st.char_idx += 1;
        st.byte_pos += ch.len_utf8();
    }

    fn rainbow_map_for_line<'a>(&self, line_idx: usize, p: &'a DrawParams) -> &'a [(usize, Color)] {
        if line_idx >= p.scroll {
            p.rainbow_maps
                .get(line_idx - p.scroll)
                .map(|v| v.as_slice())
                .unwrap_or(&[])
        } else {
            &[]
        }
    }

    pub(super) fn resolve_display_char(&self, ch: char, style: Style, app: &AppPalette) -> (char, Style) {
        if self.editor.options.list {
            let list_style = Style {
                fg: app.editor().list_chars().fg,
                ..style
            };
            match ch {
                ' ' => ('\u{00B7}', list_style),
                _ => (ch, style),
            }
        } else {
            (ch, style)
        }
    }

    fn apply_ephemeral_bg(&self, style: Style, line_idx: usize, p: &DrawParams) -> Style {
        if style.bg == Color::Reset && self.editor.ephemeral.ranges.iter().any(|r| r.covers_line(line_idx)) {
            Style {
                bg: p.ephemeral_bg,
                ..style
            }
        } else {
            style
        }
    }
}
