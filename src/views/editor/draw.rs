//! EditorView draw implementation.

use txv_core::prelude::*;
use super::EditorView;

impl EditorView {
    pub(super) fn draw_editor(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 { return; }

        // BUG 1 FIX: Clear entire editor area before drawing to prevent scroll garbage.
        let normal = Style::default();
        for y in b.y..b.y + b.h {
            surface.hline(b.x, y, b.w, ' ', normal);
        }

        let gutter_w = self.gutter_width();
        let gutter_style = Style { fg: Color::Ansi(8), ..Style::default() };
        let cursor_style = Style { attrs: Attrs { reverse: true, ..Attrs::default() }, ..Style::default() };
        let visual_style = Style { attrs: Attrs { reverse: true, ..Attrs::default() }, fg: Color::Ansi(3), ..Style::default() };

        let scroll = self.editor.viewport_scroll;
        let visual_range = self.editor.visual_range();
        let avail = b.w.saturating_sub(gutter_w) as usize;
        let wrap = self.editor.options.wrap;
        let tab_width = self.editor.options.tab_width;

        let mut row: usize = 0;
        let mut line_idx = scroll;

        while row < b.h as usize && line_idx < self.editor.buffer.line_count() {
            let y = b.y + row as u16;

            if gutter_w > 0 {
                let num = format!("{:>width$} ", line_idx + 1, width = (gutter_w - 1) as usize);
                surface.print(b.x, y, &num, gutter_style);
            }

            let line = self.editor.buffer.line(line_idx).unwrap_or_default();
            let text_x = b.x + gutter_w;
            let line_start_off = self.editor.buffer.line_col_to_offset(line_idx, 0).unwrap_or(0);
            let spans = self.highlighter.highlight_line(&line, &self.file_ext);

            let mut col_offset: usize = 0;
            let mut char_idx: usize = 0;
            let mut byte_pos = line_start_off;
            let mut visual_row = row;

            for span in &spans {
                for ch in span.text.chars() {
                    // BUG 2+3 FIX: Handle tabs specially — expand to tab_width columns.
                    if ch == '\t' {
                        let is_cursor = line_idx == self.editor.cursor_line
                            && char_idx == self.editor.cursor_col
                            && self.state.focused;
                        for ti in 0..tab_width {
                            if col_offset >= avail { break; }
                            if visual_row >= b.h as usize { break; }
                            let x = text_x + col_offset as u16;
                            let vy = b.y + visual_row as u16;
                            let st = if is_cursor && ti == 0 {
                                cursor_style
                            } else if let Some((vs, ve)) = visual_range {
                                if byte_pos >= vs && byte_pos < ve { visual_style }
                                else { span.style }
                            } else {
                                span.style
                            };
                            if self.editor.options.list {
                                let ls = Style { fg: Color::Ansi(8), ..st };
                                let c = if ti == tab_width - 1 { '\u{2192}' } else { '\u{2500}' };
                                surface.put(x, vy, c, ls);
                            } else {
                                surface.put(x, vy, ' ', st);
                            }
                            col_offset += 1;
                        }
                        char_idx += 1;
                        byte_pos += ch.len_utf8();
                        continue;
                    }

                    if wrap {
                        if col_offset >= avail {
                            col_offset = 0;
                            visual_row += 1;
                            if visual_row >= b.h as usize { break; }
                        }
                    } else if col_offset >= avail {
                        char_idx += 1;
                        byte_pos += ch.len_utf8();
                        continue;
                    }

                    if visual_row >= b.h as usize { break; }
                    let x = text_x + col_offset as u16;

                    let style = if line_idx == self.editor.cursor_line
                        && char_idx == self.editor.cursor_col
                        && self.state.focused
                    {
                        cursor_style
                    } else if let Some((vs, ve)) = visual_range {
                        if byte_pos >= vs && byte_pos < ve { visual_style } else { span.style }
                    } else {
                        span.style
                    };

                    let (display_ch, display_style) = if self.editor.options.list {
                        let list_style = Style { fg: Color::Ansi(8), ..style };
                        match ch {
                            ' ' => ('\u{00B7}', list_style),
                            _ => (ch, style),
                        }
                    } else {
                        (ch, style)
                    };

                    let vy = b.y + visual_row as u16;
                    surface.put(x, vy, display_ch, display_style);
                    col_offset += 1;
                    char_idx += 1;
                    byte_pos += ch.len_utf8();
                }
                if visual_row >= b.h as usize { break; }
            }

            if self.editor.options.list && col_offset < avail && visual_row < b.h as usize {
                let list_style = Style { fg: Color::Ansi(8), ..Style::default() };
                let vy = b.y + visual_row as u16;
                surface.put(text_x + col_offset as u16, vy, '$', list_style);
            }

            // Cursor past end of line
            if line_idx == self.editor.cursor_line && self.state.focused {
                if self.editor.cursor_col >= char_idx {
                    if visual_row < b.h as usize && col_offset < avail {
                        let cx = text_x + col_offset as u16;
                        let cy = b.y + visual_row as u16;
                        surface.put(cx, cy, ' ', cursor_style);
                    }
                }
            }

            row = visual_row + 1;
            line_idx += 1;
        }

        // Fill remaining rows with ~
        while row < b.h as usize {
            let y = b.y + row as u16;
            surface.print(b.x, y, "~", gutter_style);
            row += 1;
        }

        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let prompt_y = b.y + b.h.saturating_sub(1);
            let prompt_style = Style { attrs: Attrs { reverse: true, ..Attrs::default() }, ..Style::default() };
            surface.hline(b.x, prompt_y, b.w, ' ', prompt_style);
            let prefix = if self.editor.mode == crate::editor::keymap::EditorMode::Search { "/" } else { ":" };
            surface.print(b.x, prompt_y, prefix, prompt_style);
            surface.print(b.x + 1, prompt_y, &self.editor.command_buf, prompt_style);
        }
    }
}
