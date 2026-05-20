//! Blame gutter rendering for EditorView.

use super::EditorView;
use crate::blame::BlameState;

impl EditorView {
    /// Draw blame annotations in the gutter area (left of line numbers).
    pub(super) fn draw_blame_gutter(&mut self) {
        let Some(ref shared) = self.blame_state else {
            return;
        };
        let Ok(guard) = shared.lock() else {
            return;
        };
        let app = crate::app_palette::app_palette();
        let style = app.editor.gutter.to_style();
        let h = self.state.buffer_mut().height();
        let scroll = self.editor.viewport_scroll;

        // Collect lines to draw to avoid holding the lock while writing to buf
        let lines_to_draw: Vec<(u16, String)> = match &*guard {
            BlameState::Loading => {
                vec![(0, "loading blame...".to_string())]
            }
            BlameState::Error(_) => {
                drop(guard);
                return;
            }
            BlameState::Ready(lines) => {
                let mut result = Vec::new();
                let mut prev_hash = String::new();
                for row in 0..h as usize {
                    let line_idx = scroll + row;
                    let y = row as u16;
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
                        result.push((y, text));
                    }
                }
                result
            }
        };
        drop(guard);

        for (y, text) in &lines_to_draw {
            self.state.buffer_mut().print(0, *y, text, style);
        }
    }
}
