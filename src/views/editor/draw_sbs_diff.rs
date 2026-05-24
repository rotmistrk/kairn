//! Side-by-side diff rendering — one pane of a split diff view.

use txv_core::prelude::*;

use super::sbs_model::SbsLine;
use super::EditorView;

impl EditorView {
    pub(super) fn draw_sbs_diff(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        let sbs = match &self.sbs_state {
            Some(s) => s,
            None => return,
        };

        let max_line = sbs
            .lines
            .iter()
            .filter_map(|l| match l {
                SbsLine::Content { line_no, .. } => Some(*line_no),
                _ => None,
            })
            .max()
            .unwrap_or(0);
        let dw = digit_width(max_line);
        let gutter_w = if self.editor.options.number {
            (dw + 1) as u16
        } else {
            0
        };
        let text_x = gutter_w;
        let _avail = w.saturating_sub(gutter_w) as usize;

        let app = crate::app_palette::app_palette();
        let added_style = app.diff.added.to_style();
        let deleted_style = app.diff.deleted.to_style();
        let context_style = Style::default();
        let fold_style = app.diff.fold.to_style();
        let gap_style = txv_core::palette::palette().base.dim.to_style();

        let pal = txv_core::palette::palette();
        let cursor_style = if self.state.is_focused() {
            pal.interactive.cursor_focused.to_style()
        } else {
            pal.interactive.cursor_unfocused.to_style()
        };

        let height = h as usize;
        let scroll = sbs.scroll;
        let is_left = sbs.is_left;
        let cursor_line = sbs.cursor;

        for row in 0..height {
            let vi = scroll + row;
            let y = row as u16;
            if vi >= sbs.lines.len() {
                self.state.buffer_mut().print_line(0, y, "~", w, context_style);
                continue;
            }
            let is_cursor = vi == cursor_line;
            match &sbs.lines[vi] {
                SbsLine::Content { line_no, text, changed } => {
                    let st = if is_cursor {
                        cursor_style
                    } else if *changed {
                        if is_left {
                            deleted_style
                        } else {
                            added_style
                        }
                    } else {
                        context_style
                    };
                    if self.editor.options.number {
                        let gutter = format!("{:>width$} ", line_no + 1, width = dw);
                        let gs = crate::app_palette::app_palette().editor.gutter.to_style();
                        self.state.buffer_mut().print(0, y, &gutter, gs);
                    }
                    self.state
                        .buffer_mut()
                        .print_line(text_x, y, text, w.saturating_sub(text_x), st);
                }
                SbsLine::Gap => {
                    let st = if is_cursor {
                        cursor_style
                    } else {
                        gap_style
                    };
                    self.state.buffer_mut().print_line(0, y, "", w, st);
                }
                SbsLine::Folded { count } => {
                    let st = if is_cursor {
                        cursor_style
                    } else {
                        fold_style
                    };
                    let label = format!("--- {} lines ---", count);
                    self.state.buffer_mut().print_line(0, y, &label, w, st);
                }
            }
        }

        // Command/search prompt on last row
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let prompt_y = h.saturating_sub(1);
            let prompt_style = Style {
                attrs: Attrs {
                    reverse: true,
                    ..Attrs::default()
                },
                ..Style::default()
            };
            let prefix = if self.editor.mode == crate::editor::keymap::EditorMode::Search {
                "/"
            } else {
                ":"
            };
            let prompt_text = format!("{}{}", prefix, self.editor.command_buf);
            self.state
                .buffer_mut()
                .print_line(0, prompt_y, &prompt_text, w, prompt_style);
        }
    }
}

fn digit_width(max: usize) -> usize {
    if max < 10 {
        1
    } else if max < 100 {
        2
    } else if max < 1000 {
        3
    } else if max < 10000 {
        4
    } else {
        5
    }
}
