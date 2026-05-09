//! EditorView draw implementation.

use txv_core::prelude::*;
use super::EditorView;

impl EditorView {
    pub(super) fn draw_editor(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 { return; }
        let gutter_w = self.gutter_width();
        let gutter_style = Style { fg: Color::Ansi(8), ..Style::default() };
        let normal = Style::default();
        let cursor_style = Style { attrs: Attrs { reverse: true, ..Attrs::default() }, ..Style::default() };
        let visual_style = Style { attrs: Attrs { reverse: true, ..Attrs::default() }, fg: Color::Ansi(3), ..Style::default() };

        let scroll = self.editor.viewport_scroll;
        let visual_range = self.editor.visual_range();

        for row in 0..b.h as usize {
            let line_idx = scroll + row;
            let y = b.y + row as u16;
            surface.hline(b.x, y, b.w, ' ', normal);

            if line_idx >= self.editor.buffer.line_count() {
                surface.print(b.x, y, "~", gutter_style);
                continue;
            }

            if gutter_w > 0 {
                let num = format!("{:>width$} ", line_idx + 1, width = (gutter_w - 1) as usize);
                surface.print(b.x, y, &num, gutter_style);
            }

            let line = self.editor.buffer.line(line_idx).unwrap_or_default();
            let text_x = b.x + gutter_w;
            let avail = b.w.saturating_sub(gutter_w) as usize;
            let line_start_off = self.editor.buffer.line_col_to_offset(line_idx, 0).unwrap_or(0);
            let spans = self.highlighter.highlight_line(&line, &self.file_ext);

            let mut col_offset: u16 = 0;
            let mut byte_pos = line_start_off;
            for span in &spans {
                for ch in span.text.chars() {
                    if col_offset as usize >= avail { break; }
                    let x = text_x + col_offset;
                    let char_col = col_offset as usize;

                    let style = if line_idx == self.editor.cursor_line && char_col == self.editor.cursor_col && self.state.focused {
                        cursor_style
                    } else if let Some((vs, ve)) = visual_range {
                        if byte_pos >= vs && byte_pos < ve { visual_style } else { span.style }
                    } else { span.style };

                    let (display_ch, display_style) = if self.editor.options.list {
                        let list_style = Style { fg: Color::Ansi(8), ..style };
                        match ch {
                            ' ' => ('\u{00B7}', list_style),
                            '\t' => ('\u{2192}', list_style),
                            _ => (ch, style),
                        }
                    } else { (ch, style) };
                    surface.put(x, y, display_ch, display_style);
                    col_offset += 1;
                    byte_pos += ch.len_utf8();
                }
            }

            if self.editor.options.list && (col_offset as usize) < avail {
                let list_style = Style { fg: Color::Ansi(8), ..Style::default() };
                surface.put(text_x + col_offset, y, '$', list_style);
            }

            if line_idx == self.editor.cursor_line && self.state.focused && self.editor.cursor_col as u16 >= col_offset {
                let cx = text_x + self.editor.cursor_col as u16;
                if cx < b.x + b.w { surface.put(cx, y, ' ', cursor_style); }
            }
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
