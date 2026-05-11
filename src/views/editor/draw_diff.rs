//! Diff mode rendering — dual gutter, virtual deleted lines, fold markers.

use txv_core::prelude::*;

use super::diff_model::DiffLine;
use super::EditorView;

impl EditorView {
    pub(super) fn draw_diff(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        let ds = match &self.diff_state {
            Some(ds) => ds,
            None => return,
        };

        let max_base = self.max_base_line(ds);
        let max_buf = self.editor.buffer.line_count();
        let dw = digit_width(max_base.max(max_buf));
        // Two gutter columns: "NNN NNN " (dw + space + dw + space)
        let gutter_w = if self.editor.options.number {
            (dw * 2 + 2) as u16
        } else {
            0
        };
        let text_x = b.x + gutter_w;
        let avail = b.w.saturating_sub(gutter_w) as usize;

        let added_style = Style {
            fg: Color::Ansi(2),
            ..Style::default()
        };
        let deleted_style = Style {
            fg: Color::Ansi(1),
            ..Style::default()
        };
        let context_style = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let fold_style = Style {
            fg: Color::Ansi(5),
            ..Style::default()
        };
        let cursor_style = Style {
            attrs: Attrs {
                reverse: true,
                ..Attrs::default()
            },
            ..Style::default()
        };

        let height = b.h as usize;
        let scroll = ds.scroll;

        for row in 0..height {
            let vi = scroll + row; // virtual line index
            let y = b.y + row as u16;

            if vi >= ds.lines.len() {
                // Fill with ~
                surface.print_line(b.x, y, "~", b.w, context_style);
                continue;
            }

            let is_cursor = vi == ds.cursor && self.state.focused;
            let line = &ds.lines[vi];

            match line {
                DiffLine::Context { buf_line, base_line } => {
                    self.draw_diff_gutter(surface, b.x, y, dw, Some(*base_line), Some(*buf_line));
                    let text = self.editor.buffer.line(*buf_line).unwrap_or_default();
                    let st = if is_cursor {
                        cursor_style
                    } else {
                        context_style
                    };
                    self.draw_diff_text(surface, text_x, y, avail, &text, st);
                }
                DiffLine::Added { buf_line } => {
                    self.draw_diff_gutter(surface, b.x, y, dw, None, Some(*buf_line));
                    let text = self.editor.buffer.line(*buf_line).unwrap_or_default();
                    let st = if is_cursor {
                        cursor_style
                    } else {
                        added_style
                    };
                    self.draw_diff_text(surface, text_x, y, avail, &text, st);
                }
                DiffLine::Deleted { text, base_line } => {
                    self.draw_diff_gutter(surface, b.x, y, dw, Some(*base_line), None);
                    let st = if is_cursor {
                        cursor_style
                    } else {
                        deleted_style
                    };
                    self.draw_diff_text(surface, text_x, y, avail, text, st);
                }
                DiffLine::Folded { count } => {
                    let label = format!("--- {} lines ---", count);
                    let st = if is_cursor {
                        cursor_style
                    } else {
                        fold_style
                    };
                    surface.print_line(b.x, y, &label, b.w, st);
                }
            }
        }

        // Command/search prompt overlay
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let prompt_y = b.y + b.h.saturating_sub(1);
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
            surface.print_line(b.x, prompt_y, &prompt_text, b.w, prompt_style);
        }
    }

    fn draw_diff_gutter(
        &self,
        surface: &mut Surface,
        x: u16,
        y: u16,
        dw: usize,
        base_line: Option<usize>,
        buf_line: Option<usize>,
    ) {
        if !self.editor.options.number {
            return;
        }
        let gs = Style {
            fg: Color::Ansi(8),
            ..Style::default()
        };
        let left = match base_line {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let right = match buf_line {
            Some(n) => format!("{:>width$}", n + 1, width = dw),
            None => " ".repeat(dw),
        };
        let gutter = format!("{} {} ", left, right);
        surface.print(x, y, &gutter, gs);
    }

    fn draw_diff_text(&self, surface: &mut Surface, x: u16, y: u16, avail: usize, text: &str, style: Style) {
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
                    surface.put(x + col as u16, y, ' ', style);
                    col += 1;
                }
            } else {
                surface.put(x + col as u16, y, ch, style);
                col += display_char_width(ch) as usize;
            }
        }
        // Pad remainder
        while col < avail {
            surface.put(x + col as u16, y, ' ', style);
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
