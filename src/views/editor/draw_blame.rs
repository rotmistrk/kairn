//! Blame gutter rendering for EditorView.

use super::EditorView;
use crate::app_palette::app_palette;
use crate::blame::BlameState;
use crate::editor::keymap::EditorMode;

impl EditorView {
    /// Draw blame annotations in the gutter area (left of line numbers).
    pub(super) fn draw_blame_gutter(&mut self) {
        let Some(ref shared) = self.blame_state else {
            return;
        };
        let Ok(guard) = shared.lock() else {
            return;
        };
        let style = app_palette().editor().gutter();
        let h = self.state.buffer_mut().height();
        let scroll = self.editor.viewport_scroll();
        let max_row = self.blame_max_row(h);

        let lines_to_draw = Self::collect_blame_lines(&guard, scroll, max_row);
        drop(guard);

        for (y, text) in &lines_to_draw {
            self.state.buffer_mut().print(0, *y, text, style);
        }
    }

    fn blame_max_row(&self, h: u16) -> u16 {
        let prompt_active = self.editor.mode() == EditorMode::Command || self.editor.mode() == EditorMode::Search;
        if prompt_active {
            h.saturating_sub(1)
        } else {
            h
        }
    }

    fn collect_blame_lines(guard: &BlameState, scroll: usize, max_row: u16) -> Vec<(u16, String)> {
        match guard {
            BlameState::Loading => vec![(0, "loading blame...".to_string())],
            BlameState::Error(_) => Vec::new(),
            BlameState::Ready(lines) => {
                let mut result = Vec::new();
                let mut prev_hash = String::new();
                for row in 0..max_row as usize {
                    let line_idx = scroll + row;
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
                        result.push((row as u16, text));
                    }
                }
                result
            }
        }
    }
}
