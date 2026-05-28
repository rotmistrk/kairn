//! Diff mode rendering — dual gutter, virtual deleted lines, fold markers.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use super::diff_model::DiffLine;
use super::draw_diff_helpers::digit_width;
use super::EditorView;
use crate::app_palette::app_palette;
use crate::editor::keymap::EditorMode;

struct DiffDrawCmd {
    kind: DiffDrawKind,
    row: u16,
    is_cursor: bool,
}

/// (cursor, context, added, deleted, fold) styles for diff rendering.
type DiffRenderStyles = (Style, Style, Style, Style, Style);

enum DiffDrawKind {
    Empty,
    Context { buf_line: usize, base_line: usize },
    Added { buf_line: usize },
    Deleted { text: String, base_line: usize },
    Folded { count: usize },
}

impl EditorView {
    pub(super) fn draw_diff(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        let ds = match &self.diff_state {
            Some(ds) => ds,
            None => return,
        };

        let max_base = self.max_base_line(ds);
        let max_buf = self.editor.buf().line_count();
        let dw = digit_width(max_base.max(max_buf));
        let gutter_w = if self.editor.options.number {
            (dw * 2 + 2) as u16
        } else {
            0
        };
        let text_x = gutter_w;
        let avail = w.saturating_sub(gutter_w) as usize;

        let app = app_palette();
        let pal = palette();
        let added_style = app.diff().added();
        let deleted_style = app.diff().deleted();
        let context_style = Style::default();
        let fold_style = app.diff().fold();
        let cursor_style = if self.state.is_focused() {
            pal.style(StyleId::CursorFocused)
        } else {
            pal.style(StyleId::CursorUnfocused)
        };

        let height = h as usize;
        let (scroll, ds_cursor, ds_lines_len) = (ds.scroll, ds.cursor, ds.lines.len());

        let cmds = self.collect_diff_draw_cmds(height, scroll, ds_cursor, ds_lines_len);
        let rs = (cursor_style, context_style, added_style, deleted_style, fold_style);
        self.render_diff_cmds(&cmds, w, dw, text_x, avail, &rs);
        self.draw_diff_prompt(w, h);
    }

    #[rustfmt::skip]
    fn collect_diff_draw_cmds(
        &self, height: usize, scroll: usize, ds_cursor: usize, ds_lines_len: usize,
    ) -> Vec<DiffDrawCmd> {
        let empty = |y: u16| DiffDrawCmd { kind: DiffDrawKind::Empty, row: y, is_cursor: false };
        (0..height).map(|row| {
            let vi = scroll + row;
            let y = row as u16;
            if vi >= ds_lines_len { return empty(y); }
            let is_cursor = vi == ds_cursor && self.state.is_focused();
            let Some(ds) = self.diff_state.as_ref() else { return empty(y); };
            let Some(line) = ds.lines.get(vi) else { return empty(y); };
            let kind = match line {
                DiffLine::Context { buf_line, base_line } => {
                    DiffDrawKind::Context { buf_line: *buf_line, base_line: *base_line }
                }
                DiffLine::Added { buf_line } => DiffDrawKind::Added { buf_line: *buf_line },
                DiffLine::Deleted { text, base_line } => {
                    DiffDrawKind::Deleted { text: text.clone(), base_line: *base_line }
                }
                DiffLine::Folded { count } => DiffDrawKind::Folded { count: *count },
            };
            DiffDrawCmd { kind, row: y, is_cursor }
        }).collect()
    }

    fn render_diff_cmds(
        &mut self,
        cmds: &[DiffDrawCmd],
        w: u16,
        dw: usize,
        text_x: u16,
        avail: usize,
        rs: &DiffRenderStyles,
    ) {
        for cmd in cmds {
            self.render_one_diff_cmd(cmd, w, dw, text_x, avail, rs);
        }
    }

    #[rustfmt::skip]
    fn render_one_diff_cmd(
        &mut self, cmd: &DiffDrawCmd, w: u16, dw: usize, text_x: u16, avail: usize,
        rs: &DiffRenderStyles,
    ) {
        let y = cmd.row;
        let &(cursor_style, context_style, added_style, deleted_style, fold_style) = rs;
        match &cmd.kind {
            DiffDrawKind::Empty => {
                self.state.buffer_mut().print_line(0, y, "~", w, context_style);
            }
            DiffDrawKind::Context { buf_line, base_line } => {
                self.draw_diff_gutter(0, y, dw, Some(*base_line), Some(*buf_line));
                let text = self.editor.buf().line(*buf_line).unwrap_or_default();
                let st = if cmd.is_cursor { cursor_style } else { context_style };
                self.draw_diff_text(text_x, y, avail, &text, st);
            }
            DiffDrawKind::Added { buf_line } => {
                self.draw_diff_gutter(0, y, dw, None, Some(*buf_line));
                let text = self.editor.buf().line(*buf_line).unwrap_or_default();
                let st = if cmd.is_cursor { cursor_style } else { added_style };
                self.draw_diff_text(text_x, y, avail, &text, st);
            }
            DiffDrawKind::Deleted { text, base_line } => {
                self.draw_diff_gutter(0, y, dw, Some(*base_line), None);
                let st = if cmd.is_cursor { cursor_style } else { deleted_style };
                self.draw_diff_text(text_x, y, avail, text, st);
            }
            DiffDrawKind::Folded { count } => {
                let label = format!("--- {} lines ---", count);
                let st = if cmd.is_cursor { cursor_style } else { fold_style };
                self.state.buffer_mut().print_line(0, y, &label, w, st);
            }
        }
    }

    fn draw_diff_prompt(&mut self, w: u16, h: u16) {
        if self.editor.mode == EditorMode::Command || self.editor.mode == EditorMode::Search {
            let prompt_y = h.saturating_sub(1);
            let prompt_style = palette().style(StyleId::StatusBar);
            let prefix = if self.editor.mode == EditorMode::Search {
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
