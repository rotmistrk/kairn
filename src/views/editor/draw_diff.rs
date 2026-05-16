//! Diff mode rendering — dual gutter, virtual deleted lines, fold markers.

use txv_core::prelude::*;

use super::diff_model::DiffLine;
use super::EditorView;

impl EditorView {
    pub(super) fn draw_diff(&mut self) {
        let w = self.state.buf.width();
        let h = self.state.buf.height();
        let ds = match &self.diff_state {
            Some(ds) => ds,
            None => return,
        };

        let max_base = self.max_base_line(ds);
        let max_buf = self.editor.buf().line_count();
        let dw = digit_width(max_base.max(max_buf));
        // Two gutter columns: "NNN NNN " (dw + space + dw + space)
        let gutter_w = if self.editor.options.number {
            (dw * 2 + 2) as u16
        } else {
            0
        };
        let text_x = gutter_w;
        let avail = w.saturating_sub(gutter_w) as usize;

        let app = crate::app_palette::app_palette();
        let pal = txv_core::palette::palette();
        let added_style = app.diff.added.to_style();
        let deleted_style = app.diff.deleted.to_style();
        let context_style = Style::default();
        let fold_style = app.diff.fold.to_style();
        let cursor_style = if self.state.is_focused() {
            pal.interactive.cursor_focused.to_style()
        } else {
            pal.interactive.cursor_unfocused.to_style()
        };

        let height = h as usize;
        let scroll = ds.scroll;
        let ds_cursor = ds.cursor;
        let ds_lines_len = ds.lines.len();

        // Collect draw commands to avoid borrow issues with self.diff_state
        struct DrawCmd {
            kind: DrawKind,
            row: u16,
            is_cursor: bool,
        }
        enum DrawKind {
            Empty,
            Context { buf_line: usize, base_line: usize },
            Added { buf_line: usize },
            Deleted { text: String, base_line: usize },
            Folded { count: usize },
        }

        let cmds: Vec<DrawCmd> = (0..height)
            .map(|row| {
                let vi = scroll + row;
                let y = row as u16;
                if vi >= ds_lines_len {
                    return DrawCmd {
                        kind: DrawKind::Empty,
                        row: y,
                        is_cursor: false,
                    };
                }
                let is_cursor = vi == ds_cursor && self.state.is_focused();
                let Some(ds) = self.diff_state.as_ref() else {
                    return DrawCmd {
                        kind: DrawKind::Empty,
                        row: y,
                        is_cursor: false,
                    };
                };
                let Some(line) = ds.lines.get(vi) else {
                    return DrawCmd {
                        kind: DrawKind::Empty,
                        row: y,
                        is_cursor: false,
                    };
                };
                let kind = match line {
                    DiffLine::Context { buf_line, base_line } => DrawKind::Context {
                        buf_line: *buf_line,
                        base_line: *base_line,
                    },
                    DiffLine::Added { buf_line } => DrawKind::Added { buf_line: *buf_line },
                    DiffLine::Deleted { text, base_line } => DrawKind::Deleted {
                        text: text.clone(),
                        base_line: *base_line,
                    },
                    DiffLine::Folded { count } => DrawKind::Folded { count: *count },
                };
                DrawCmd {
                    kind,
                    row: y,
                    is_cursor,
                }
            })
            .collect();

        for cmd in &cmds {
            let y = cmd.row;
            match &cmd.kind {
                DrawKind::Empty => {
                    self.state.buf.print_line(0, y, "~", w, context_style);
                }
                DrawKind::Context { buf_line, base_line } => {
                    self.draw_diff_gutter(0, y, dw, Some(*base_line), Some(*buf_line));
                    let text = self.editor.buf().line(*buf_line).unwrap_or_default();
                    let st = if cmd.is_cursor {
                        cursor_style
                    } else {
                        context_style
                    };
                    self.draw_diff_text(text_x, y, avail, &text, st);
                }
                DrawKind::Added { buf_line } => {
                    self.draw_diff_gutter(0, y, dw, None, Some(*buf_line));
                    let text = self.editor.buf().line(*buf_line).unwrap_or_default();
                    let st = if cmd.is_cursor {
                        cursor_style
                    } else {
                        added_style
                    };
                    self.draw_diff_text(text_x, y, avail, &text, st);
                }
                DrawKind::Deleted { text, base_line } => {
                    self.draw_diff_gutter(0, y, dw, Some(*base_line), None);
                    let st = if cmd.is_cursor {
                        cursor_style
                    } else {
                        deleted_style
                    };
                    self.draw_diff_text(text_x, y, avail, text, st);
                }
                DrawKind::Folded { count } => {
                    let label = format!("--- {} lines ---", count);
                    let st = if cmd.is_cursor {
                        cursor_style
                    } else {
                        fold_style
                    };
                    self.state.buf.print_line(0, y, &label, w, st);
                }
            }
        }

        // Command/search prompt overlay
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
            self.state.buf.print_line(0, prompt_y, &prompt_text, w, prompt_style);
        }
    }

    fn draw_diff_gutter(&mut self, x: u16, y: u16, dw: usize, base_line: Option<usize>, buf_line: Option<usize>) {
        if !self.editor.options.number {
            return;
        }
        let gs = crate::app_palette::app_palette().editor.gutter.to_style();
        let left = match base_line {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let right = match buf_line {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let gutter = format!("{} {} ", left, right);
        self.state.buf.print(x, y, &gutter, gs);
    }

    fn draw_diff_text(&mut self, x: u16, y: u16, avail: usize, text: &str, style: Style) {
        use txv_core::text::display_char_width;
        let tab_w = self.editor.options.tab_width;
        let mut col = 0usize;
        for ch in text.chars() {
            if col >= avail {
                break;
            }
            if ch == '\t' {
                let spaces = tab_w - (col % tab_w);
                for _ in 0..spaces {
                    if col >= avail {
                        break;
                    }
                    self.state.buf.put(x + col as u16, y, ' ', style);
                    col += 1;
                }
            } else {
                self.state.buf.put(x + col as u16, y, ch, style);
                col += display_char_width(ch) as usize;
            }
        }
        // Pad remainder
        while col < avail {
            self.state.buf.put(x + col as u16, y, ' ', style);
            col += 1;
        }
    }

    fn max_base_line(&self, ds: &super::diff_model::DiffState) -> usize {
        ds.lines
            .iter()
            .filter_map(|l| match l {
                DiffLine::Context { base_line, .. } | DiffLine::Deleted { base_line, .. } => Some(*base_line),
                _ => None,
            })
            .max()
            .unwrap_or(0)
            + 1
    }
}

fn digit_width(n: usize) -> usize {
    if n == 0 {
        1
    } else {
        (n as f64).log10() as usize + 1
    }
}
