//! Blame gutter rendering for EditorView.

use txv_core::prelude::*;

use super::EditorView;
use crate::blame::BlameState;

impl EditorView {
    /// Draw blame annotations in the gutter area (left of line numbers).
    pub(super) fn draw_blame_gutter(&self, surface: &mut Surface) {
        let Some(ref shared) = self.blame_state else {
            return;
        };
        let Ok(guard) = shared.lock() else {
            return;
        };
        let app = crate::app_palette::app_palette();
        let style = app.editor.gutter.to_style();
        let lines = match &*guard {
            BlameState::Ready(lines) => lines,
            BlameState::Loading => {
                let b = self.state.bounds();
                surface.print(b.x, b.y, "loading blame...", style);
                return;
            }
            BlameState::Error(_) => return,
        };

        let b = self.state.bounds();
        let scroll = self.editor.viewport_scroll;
        let mut prev_hash = String::new();

        for row in 0..b.h as usize {
            let line_idx = scroll + row;
            let y = b.y + row as u16;
            let blame = lines.iter().find(|bl| bl.line == line_idx);
            let text = match blame {
                Some(bl) => {
                    if bl.hash == prev_hash {
                        format!("{:23}", "│")
                    } else {
                        prev_hash = bl.hash.clone();
                        format!("{} {:10} {} ", bl.hash, bl.author, &bl.date[5..])
                    }
                }
                None => {
                    prev_hash.clear();
                    String::new()
                }
            };
            if !text.is_empty() {
                surface.print(b.x, y, &text, style);
            }
        }
    }
}
