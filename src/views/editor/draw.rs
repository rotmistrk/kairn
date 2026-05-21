//! EditorView draw implementation.

use txv_core::prelude::*;

use super::{draw_style::char_style, EditorView};
use crate::highlight::HlSpan;

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

        let pal = txv_core::palette::palette();
        let app = crate::app_palette::app_palette();
        let normal = Style::default();
        let gutter_w = self.gutter_width();
        let gutter_style = app.editor.gutter.to_style();
        let cursor_style = app.editor.cursor.to_style();
        let hl_match_style = app.editor.highlight_match.to_style();
        let hl_other_bg = app.editor.highlight_other.to_style().bg;
        let visual_bg = if self.state.is_focused() {
            pal.interactive.visual_selection.bg.unwrap_or(Color::Ansi(4))
        } else {
            pal.interactive.cursor_unfocused.bg.unwrap_or(Color::Ansi(8))
        };

        let scroll = self.editor.viewport_scroll;
        let visual_range = self.editor.visual_range();
        let avail = w.saturating_sub(gutter_w) as usize;
        let wrap = self.editor.options.wrap;
        let h_off = if wrap {
            0
        } else {
            self.editor.h_scroll
        };
        let tab_width = self.editor.options.tab_width;
        let highlight = self.editor.highlight.as_ref();

        let mut row: usize = 0;
        let mut line_idx = scroll;

        // Pre-compute highlighted spans for the visible viewport using cached state.
        let total_lines = self.editor.buf().line_count();
        let viewport_end = (scroll + h as usize).min(total_lines);
        let viewport_spans = {
            let mut cache = self.hl_cache.borrow_mut();
            cache.highlight_viewport(
                scroll,
                viewport_end,
                total_lines,
                |i| self.editor.buf().line(i).unwrap_or_default(),
                self.highlighter.syntax_set(),
                self.highlighter.theme(),
            )
        };

        // Matchparen: find matching bracket for cursor position.
        let matchparen_pos = if self.editor.options.matchparen {
            crate::editor::motions::match_bracket(&self.editor.buf(), self.editor.cursor_line, self.editor.cursor_col)
        } else {
            None
        };
        let matchparen_style = app.editor.matchparen;
        let rainbow_map = if self.editor.options.rainbow {
            super::draw_style::rainbow_brackets(&self.editor.buf().line(self.editor.cursor_line).unwrap_or_default())
        } else {
            Vec::new()
        };

        while row < h as usize && line_idx < viewport_end {
            let y = row as u16;
            let text_x = gutter_w;

            // --- Gutter ---
            if gutter_w > 0 {
                let num = format!("{:>width$} ", line_idx + 1, width = (gutter_w - 1) as usize);
                self.state.buffer_mut().print(0, y, &num, gutter_style);
                // Diagnostic marker in last gutter column
                if let Some(sev) = self.diagnostic_severity_at(line_idx) {
                    let marker_style = crate::views::editor::draw_diagnostics::diag_marker_style(sev);
                    self.state.buffer_mut().put(gutter_w - 1, y, '●', marker_style);
                }
            }

            // --- Line content: write char-by-char, then pad to full width ---
            let line = self.editor.buf().line(line_idx).unwrap_or_default();
            let line_start_off = self.editor.buf().line_col_to_offset(line_idx, 0).unwrap_or(0);
            let default_spans;
            let spans: &[HlSpan] = match viewport_spans.get(line_idx - scroll) {
                Some(s) => s,
                None => {
                    default_spans = [HlSpan::plain(line.clone())];
                    &default_spans
                }
            };

            let mut col_offset: usize = 0;
            let mut char_idx: usize = 0;
            let mut byte_pos = line_start_off;
            let mut visual_row = row;

            for span in spans {
                for ch in span.text.chars() {
                    if ch == '\t' {
                        let st = char_style(
                            span.style,
                            byte_pos,
                            visual_range,
                            visual_bg,
                            highlight,
                            hl_match_style,
                            hl_other_bg,
                        );
                        for ti in 0..tab_width {
                            if col_offset >= h_off + avail || visual_row >= h as usize {
                                break;
                            }
                            if col_offset >= h_off {
                                let x = text_x + (col_offset - h_off) as u16;
                                let vy = visual_row as u16;
                                if self.editor.options.list {
                                    let ls = app.editor.list_chars.resolve(&st);
                                    let c = if ti == tab_width - 1 {
                                        '\u{2192}'
                                    } else {
                                        '\u{2500}'
                                    };
                                    self.state.buffer_mut().put(x, vy, c, ls);
                                } else {
                                    self.state.buffer_mut().put(x, vy, ' ', st);
                                }
                            }
                            col_offset += 1;
                        }
                        char_idx += 1;
                        byte_pos += ch.len_utf8();
                        continue;
                    }

                    if wrap {
                        if col_offset >= avail {
                            let vy = visual_row as u16;
                            for pad_col in col_offset..avail {
                                self.state.buffer_mut().put(text_x + pad_col as u16, vy, ' ', normal);
                            }
                            col_offset = 0;
                            visual_row += 1;
                            if visual_row >= h as usize {
                                break;
                            }
                            if gutter_w > 0 {
                                let wy = visual_row as u16;
                                self.state.buffer_mut().print_line(0, wy, "", gutter_w, gutter_style);
                            }
                        }
                    } else if col_offset >= h_off + avail {
                        char_idx += 1;
                        byte_pos += ch.len_utf8();
                        continue;
                    }

                    if visual_row >= h as usize {
                        break;
                    }
                    if col_offset < h_off {
                        col_offset += display_char_width(ch) as usize;
                        char_idx += 1;
                        byte_pos += ch.len_utf8();
                        continue;
                    }
                    let x = text_x + (col_offset - h_off) as u16;
                    let style = char_style(
                        span.style,
                        byte_pos,
                        visual_range,
                        visual_bg,
                        highlight,
                        hl_match_style,
                        hl_other_bg,
                    );

                    let (display_ch, display_style) = if self.editor.options.list {
                        let list_style = app.editor.list_chars.resolve(&style);
                        match ch {
                            ' ' => ('\u{00B7}', list_style),
                            _ => (ch, style),
                        }
                    } else {
                        (ch, style)
                    };

                    let vy = visual_row as u16;
                    let display_style = super::draw_style::bracket_overlay(
                        display_style,
                        line_idx,
                        char_idx,
                        self.editor.cursor_line,
                        matchparen_pos,
                        &matchparen_style,
                        &rainbow_map,
                    );
                    self.state.buffer_mut().put(x, vy, display_ch, display_style);
                    col_offset += display_char_width(ch) as usize;
                    char_idx += 1;
                    byte_pos += ch.len_utf8();
                }
                if visual_row >= h as usize {
                    break;
                }
            }

            // End-of-line marker in list mode
            if self.editor.options.list && col_offset >= h_off && col_offset - h_off < avail && visual_row < h as usize
            {
                let list_style = app.editor.list_chars.to_style();
                let x = text_x + (col_offset - h_off) as u16;
                self.state.buffer_mut().put(x, visual_row as u16, '$', list_style);
                col_offset += 1;
            }

            // --- PAD remainder + indent guides ---
            if visual_row < h as usize {
                let vy = visual_row as u16;
                for pad_col in col_offset..avail {
                    self.state.buffer_mut().put(text_x + pad_col as u16, vy, ' ', normal);
                }
                if self.editor.options.guides {
                    super::draw_style::draw_indent_guides(
                        self.state.buffer_mut(),
                        &line,
                        text_x,
                        vy,
                        tab_width,
                        avail,
                        gutter_style,
                    );
                }
            }

            // --- Cursor rendering (AFTER content, reverse style, tab-aware) ---
            if line_idx == self.editor.cursor_line && self.state.is_focused() {
                // Compute cursor visual position accounting for wrapping
                let (cursor_vrow, cursor_vcol) =
                    self.cursor_visual_pos(line_idx, self.editor.cursor_col, avail + h_off, tab_width, row);
                let screen_col = cursor_vcol.saturating_sub(h_off);
                if cursor_vrow < h as usize && cursor_vcol >= h_off && screen_col < avail {
                    let cx = text_x + screen_col as u16;
                    let cy = cursor_vrow as u16;
                    let under = self.state.buffer_mut().cell(cx, cy).ch;
                    self.state.buffer_mut().put(cx, cy, under, cursor_style);
                }
            }

            row = visual_row + 1;
            line_idx += 1;
        }

        // Highlight word (gs target) — paint bg on the word only
        if let Some((hl_line, col_start, col_end)) = self.highlight_word {
            self.paint_highlight_word(hl_line, col_start, col_end, scroll);
        }

        // Fill remaining rows + prompt
        self.draw_footer(row, gutter_style);
    }
}
