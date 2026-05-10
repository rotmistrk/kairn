//! EditorView draw implementation.

use txv_core::prelude::*;

use super::EditorView;

impl EditorView {
    pub(super) fn draw_editor(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 { return; }

        let normal = Style::default();
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
            let text_x = b.x + gutter_w;

            // --- Gutter ---
            if gutter_w > 0 {
                let num = format!("{:>width$} ", line_idx + 1, width = (gutter_w - 1) as usize);
                surface.print(b.x, y, &num, gutter_style);
            }

            // --- Line content: write char-by-char, then pad to full width ---
            let line = self.editor.buffer.line(line_idx).unwrap_or_default();
            let line_start_off = self.editor.buffer.line_col_to_offset(line_idx, 0).unwrap_or(0);
            let spans = self.highlighter.highlight_line(&line, &self.file_ext);

            let mut col_offset: usize = 0;
            let mut char_idx: usize = 0;
            let mut byte_pos = line_start_off;
            let mut visual_row = row;

            for span in &spans {
                for ch in span.text.chars() {
                    if ch == '\t' {
                        for ti in 0..tab_width {
                            if col_offset >= avail { break; }
                            if visual_row >= b.h as usize { break; }
                            let x = text_x + col_offset as u16;
                            let vy = b.y + visual_row as u16;
                            let st = if let Some((vs, ve)) = visual_range {
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
                            // Pad remainder of current visual row
                            let vy = b.y + visual_row as u16;
                            for pad_col in col_offset..avail {
                                surface.put(text_x + pad_col as u16, vy, ' ', normal);
                            }
                            col_offset = 0;
                            visual_row += 1;
                            if visual_row >= b.h as usize { break; }
                            // Gutter for wrapped line (blank)
                            if gutter_w > 0 {
                                let wy = b.y + visual_row as u16;
                                surface.print_line(b.x, wy, "", gutter_w, gutter_style);
                            }
                        }
                    } else if col_offset >= avail {
                        char_idx += 1;
                        byte_pos += ch.len_utf8();
                        continue;
                    }

                    if visual_row >= b.h as usize { break; }
                    let x = text_x + col_offset as u16;

                    let style = if let Some((vs, ve)) = visual_range {
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

            // End-of-line marker in list mode
            if self.editor.options.list && col_offset < avail && visual_row < b.h as usize {
                let list_style = Style { fg: Color::Ansi(8), ..Style::default() };
                let vy = b.y + visual_row as u16;
                let x = text_x + col_offset as u16;
                surface.put(x, vy, '$', list_style);
                col_offset += 1;
            }

            // --- PAD remainder of line to full width (TXV model) ---
            if visual_row < b.h as usize {
                let vy = b.y + visual_row as u16;
                for pad_col in col_offset..avail {
                    surface.put(text_x + pad_col as u16, vy, ' ', normal);
                }
            }

            // --- Cursor rendering (AFTER content, reverse style, tab-aware) ---
            if line_idx == self.editor.cursor_line && self.state.focused {
                let cursor_visual_col = if self.editor.cursor_col >= char_idx {
                    col_offset
                } else {
                    let line_ref = self.editor.buffer.line(line_idx).unwrap_or_default();
                    let mut vcol: usize = 0;
                    for (ci, ch) in line_ref.chars().enumerate() {
                        if ci == self.editor.cursor_col { break; }
                        if ch == '\t' { vcol += tab_width; } else { vcol += 1; }
                    }
                    vcol
                };
                if visual_row < b.h as usize && cursor_visual_col < avail {
                    let cx = text_x + cursor_visual_col as u16;
                    let cy = b.y + visual_row as u16;
                    let under = surface.cell(cx, cy).ch;
                    surface.put(cx, cy, under, cursor_style);
                }
            }

            row = visual_row + 1;
            line_idx += 1;
        }

        // Fill remaining rows with ~ (full-width)
        while row < b.h as usize {
            let y = b.y + row as u16;
            let mut tilde = String::with_capacity(b.w as usize);
            tilde.push('~');
            surface.print_line(b.x, y, &tilde, b.w, gutter_style);
            row += 1;
        }

        // Command/search prompt (full-width)
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let prompt_y = b.y + b.h.saturating_sub(1);
            let prompt_style = Style { attrs: Attrs { reverse: true, ..Attrs::default() }, ..Style::default() };
            let prefix = if self.editor.mode == crate::editor::keymap::EditorMode::Search { "/" } else { ":" };
            let prompt_text = format!("{}{}", prefix, self.editor.command_buf);
            surface.print_line(b.x, prompt_y, &prompt_text, b.w, prompt_style);
        }
    }
}
