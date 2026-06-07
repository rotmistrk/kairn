//! Hardware cursor support — mode-dependent cursor shape.

use crate::editor::keymap::EditorMode;
use crate::settings::CursorStyle;

use super::EditorView;

impl EditorView {
    /// Determine cursor style for the current editor mode.
    pub(super) fn cursor_style_for_mode(&self) -> CursorStyle {
        match self.editor.mode {
            EditorMode::Insert => self.editor.options.cursor_insert,
            EditorMode::Normal | EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock => {
                self.editor.options.cursor_normal
            }
            EditorMode::Command | EditorMode::Search => self.editor.options.cursor_command,
        }
    }

    /// Compute hardware cursor screen position (relative to view bounds).
    /// Returns None if cursor is off-screen.
    pub(super) fn hw_cursor_screen_pos(&self) -> Option<(u16, u16)> {
        let scroll = self.editor.viewport_scroll;
        let h = self.state.bounds().h() as usize;
        let avail = self.text_avail_width();
        let tab_w = self.editor.options.tab_width;
        let gutter_w = self.gutter_width();
        let h_off = if self.editor.options.wrap {
            0
        } else {
            self.editor.h_scroll
        };

        // Compute visual row of cursor line
        let mut vis_row: usize = 0;
        for li in scroll..self.editor.cursor_line {
            vis_row += if self.editor.options.wrap {
                self.wrapped_line_rows(li, avail)
            } else {
                1
            };
        }
        let (vrow, vcol) = self.cursor_visual_pos(
            self.editor.cursor_line,
            self.editor.cursor_col,
            avail + h_off,
            tab_w,
            vis_row,
        );

        if vrow >= h {
            return None;
        }
        let screen_col = vcol.checked_sub(h_off)?;
        if screen_col >= avail {
            return None;
        }
        Some((gutter_w + screen_col as u16, vrow as u16))
    }

    /// Whether the current mode uses a hardware cursor (skip software cursor drawing).
    pub(super) fn uses_hw_cursor(&self) -> bool {
        self.cursor_style_for_mode() != CursorStyle::Software
    }
}
