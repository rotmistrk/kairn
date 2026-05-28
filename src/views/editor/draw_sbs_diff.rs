//! Side-by-side diff rendering — both columns in a single view.

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

        let app = crate::app_palette::app_palette();
        let added_style = app.diff().added();
        let deleted_style = app.diff().deleted();
        let context_style = Style::default();
        let fold_style = app.diff().fold();
        let gap_style = txv_core::palette::palette().base().dim();
        let divider_style = txv_core::palette::palette().base().dim();
        let gutter_style = app.editor().gutter();

        let pal = txv_core::palette::palette();
        let cursor_style = if self.state.is_focused() {
            pal.interactive().cursor_focused()
        } else {
            pal.interactive().cursor_unfocused()
        };

        // Compute gutter width from max line number
        let show_numbers = self.editor.options.number;
        let dw = if show_numbers {
            let max_line = sbs
                .left
                .iter()
                .chain(sbs.right.iter())
                .filter_map(|l| match l {
                    SbsLine::Content { line_no, .. } => Some(*line_no),
                    _ => None,
                })
                .max()
                .unwrap_or(0);
            digit_width(max_line)
        } else {
            0
        };
        let gutter_w = if show_numbers {
            (dw + 1) as u16
        } else {
            0
        };

        // Layout: [gutter|left_text] | [gutter|right_text]
        let half_w = w.saturating_sub(1) / 2; // -1 for divider
        let right_x = half_w + 1;
        let right_w = w.saturating_sub(right_x);

        let height = h as usize;
        let scroll = sbs.scroll;
        let cursor_line = sbs.cursor;
        let g = txv_core::glyphs::glyphs();

        for row in 0..height {
            let vi = scroll + row;
            let y = row as u16;

            // Draw divider
            self.state.buffer_mut().put(half_w, y, g.ui.separator_v, divider_style);

            if vi >= sbs.left.len() {
                self.state.buffer_mut().print_line(0, y, "~", half_w, context_style);
                self.state
                    .buffer_mut()
                    .print_line(right_x, y, "~", right_w, context_style);
                continue;
            }

            let is_cursor = vi == cursor_line;

            // Left column
            draw_sbs_line(
                self.state.buffer_mut(),
                &sbs.left[vi],
                0,
                half_w,
                y,
                is_cursor,
                true,
                gutter_w,
                dw,
                gutter_style,
                cursor_style,
                deleted_style,
                added_style,
                context_style,
                gap_style,
                fold_style,
            );

            // Right column
            let right_line = sbs.right.get(vi).unwrap_or(&SbsLine::Gap);
            draw_sbs_line(
                self.state.buffer_mut(),
                right_line,
                right_x,
                right_w,
                y,
                is_cursor,
                false,
                gutter_w,
                dw,
                gutter_style,
                cursor_style,
                deleted_style,
                added_style,
                context_style,
                gap_style,
                fold_style,
            );
        }

        // Command/search prompt on last row
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let prompt_y = h.saturating_sub(1);
            let prompt_style = txv_core::palette::palette().chrome().status_bar();
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

#[allow(clippy::too_many_arguments)]
fn draw_sbs_line(
    buf: &mut Buffer,
    line: &SbsLine,
    x: u16,
    w: u16,
    y: u16,
    is_cursor: bool,
    is_left: bool,
    gutter_w: u16,
    dw: usize,
    gutter_style: Style,
    cursor_style: Style,
    deleted_style: Style,
    added_style: Style,
    context_style: Style,
    gap_style: Style,
    fold_style: Style,
) {
    let text_x = x + gutter_w;
    let text_w = w.saturating_sub(gutter_w);

    match line {
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
            if gutter_w > 0 {
                let gutter = format!("{:>width$} ", line_no + 1, width = dw);
                buf.print(x, y, &gutter, gutter_style);
            }
            buf.print_line(text_x, y, text, text_w, st);
        }
        SbsLine::Gap => {
            let st = if is_cursor {
                cursor_style
            } else {
                gap_style
            };
            buf.print_line(x, y, "", w, st);
        }
        SbsLine::Folded { count } => {
            let st = if is_cursor {
                cursor_style
            } else {
                fold_style
            };
            let label = format!("--- {} lines ---", count);
            buf.print_line(x, y, &label, w, st);
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
