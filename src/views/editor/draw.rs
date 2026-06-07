//! EditorView draw implementation.

use super::delegate::KairnEditorDelegate;
use super::EditorView;
use crate::app_palette::app_palette;
use txv_edit::view::draw::draw_editor;

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
        self.draw_normal_mode();
    }

    /// Normal-mode rendering: delegate to txv-edit engine + post-draw overlays.
    fn draw_normal_mode(&mut self) {
        let delegate = KairnEditorDelegate {
            gutter_signs: &self.gutter_signs,
            diagnostics: self.diagnostics.as_deref(),
            show_gutter_signs: self.editor.options().gutter_signs() && self.editor.options().number(),
            blame_w: if self.blame_state.is_some() {
                24
            } else {
                0
            },
            number: self.editor.options().number(),
        };
        {
            let mut hl_cache = self.hl_cache.borrow_mut();
            draw_editor(
                self.state.buffer_mut(),
                &self.editor,
                &delegate,
                &mut hl_cache,
                &self.highlighter,
            );
        }
        if let Some((hl_line, col_start, col_end)) = self.highlight_word {
            let scroll = self.editor.viewport_scroll();
            self.paint_highlight_word(hl_line, col_start, col_end, scroll);
        }
        self.draw_line_cursor_software();
    }

    /// Paint software cursor (reverse-video block) when not using hardware cursor.
    fn draw_line_cursor_software(&mut self) {
        if !self.state.is_focused() || self.uses_hw_cursor() {
            return;
        }
        let cursor_style = app_palette().editor().cursor();
        let gutter_w = self.gutter_width();
        let scroll = self.editor.viewport_scroll();
        let line = self.editor.cursor_line();
        if line < scroll {
            return;
        }
        let h = self.state.buffer_mut().height() as usize;
        let avail = self.text_avail_width();
        let tab_w = self.editor.options().tab_width();
        let h_off = if self.editor.options().wrap() {
            0
        } else {
            self.editor.h_scroll()
        };

        let mut vis_row: usize = 0;
        for li in scroll..line {
            vis_row += if self.editor.options().wrap() {
                self.wrapped_line_rows(li, avail)
            } else {
                1
            };
        }
        let (vrow, vcol) = self.cursor_visual_pos(line, self.editor.cursor_col(), avail + h_off, tab_w, vis_row);
        let screen_col = vcol.saturating_sub(h_off);
        if vrow < h && vcol >= h_off && screen_col < avail {
            let cx = gutter_w + screen_col as u16;
            let cy = vrow as u16;
            let under = self.state.buffer_mut().cell(cx, cy).ch();
            self.state.buffer_mut().put(cx, cy, under, cursor_style);
        }
    }
}
