//! Style helpers for editor draw — computes per-character style overlays.

use txv_core::palette::PaletteStyle;
use txv_core::prelude::{Attrs, Color, Style};

/// Compute the effective style for a character at `byte_pos`, applying
/// visual selection and search highlight overlays.
pub(super) fn char_style(
    base: Style,
    byte_pos: usize,
    visual_range: Option<(usize, usize)>,
    visual_bg: Color,
    highlight: Option<&crate::editor::highlight_state::HighlightState>,
    hl_match: Style,
    hl_other_bg: Color,
) -> Style {
    if let Some((vs, ve)) = visual_range {
        if byte_pos >= vs && byte_pos < ve {
            return Style { bg: visual_bg, ..base };
        }
    } else if let Some(is_current) = highlight.and_then(|h| h.match_at(byte_pos)) {
        if is_current {
            return hl_match;
        }
        return Style {
            bg: hl_other_bg,
            ..base
        };
    }
    base
}

/// Apply matchparen and rainbow bracket overlays to a character style.
pub(super) fn bracket_overlay(
    base: Style,
    line_idx: usize,
    char_idx: usize,
    cursor_line: usize,
    matchparen_pos: Option<(usize, usize)>,
    matchparen_style: &PaletteStyle,
    rainbow_map: &[(usize, Color)],
) -> Style {
    if matchparen_pos == Some((line_idx, char_idx)) {
        matchparen_style.resolve(&base)
    } else if line_idx == cursor_line {
        if let Some(&(_, color)) = rainbow_map.iter().find(|(col, _)| *col == char_idx) {
            Style { fg: color, ..base }
        } else {
            base
        }
    } else {
        base
    }
}

/// Draw indent guides for a line. Draws `│` at each tab_width column within the indent.
pub(super) fn draw_indent_guides(
    buf: &mut txv_core::buffer::Buffer,
    line: &str,
    text_x: u16,
    vy: u16,
    tab_width: usize,
    avail: usize,
    style: Style,
) {
    let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
    let indent_visual = if line.starts_with('\t') {
        indent * tab_width
    } else {
        indent
    };
    let mut g = tab_width;
    while g < indent_visual && g < avail {
        buf.put(text_x + g as u16, vy, '\u{2502}', style);
        g += tab_width;
    }
}

/// Rainbow bracket colors — 4 distinct hues cycling by depth.
const RAINBOW_COLORS: [Color; 4] = [
    Color::Ansi(3), // yellow
    Color::Ansi(5), // magenta
    Color::Ansi(6), // cyan
    Color::Ansi(2), // green
];

/// Compute rainbow bracket colors for a line. Returns a vec of (char_index, color)
/// for each bracket character.
pub(super) fn rainbow_brackets(line: &str) -> Vec<(usize, Color)> {
    let mut result = Vec::new();
    let mut depth: usize = 0;
    for (idx, ch) in line.chars().enumerate() {
        match ch {
            '(' | '[' | '{' | '<' => {
                result.push((idx, RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]));
                depth += 1;
            }
            ')' | ']' | '}' | '>' => {
                depth = depth.saturating_sub(1);
                result.push((idx, RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]));
            }
            _ => {}
        }
    }
    result
}

impl super::EditorView {
    /// Draw tilde fill and command/search prompt at the bottom.
    pub(super) fn draw_footer(&mut self, mut row: usize, gutter_style: Style) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        while row < h as usize {
            let y = row as u16;
            self.state.buffer_mut().print_line(0, y, "~", w, gutter_style);
            row += 1;
        }
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

/// Paint highlight background on a single row (for gs target line).
pub(super) fn paint_line_bg(buf: &mut txv_core::buffer::Buffer, y: u16, from_x: u16, to_x: u16) {
    let Some(bg) = txv_core::palette::palette().interactive.search_match.bg else {
        return;
    };
    let bw = buf.width() as usize;
    let cells = buf.cells_mut();
    let base = y as usize * bw;
    for x in (from_x as usize)..(to_x as usize) {
        if let Some(c) = cells.get_mut(base + x) {
            c.style.bg = bg;
        }
    }
}

impl super::EditorView {
    /// Paint highlight on a word (gs target), accounting for wrapped lines.
    pub(super) fn paint_highlight_word(&mut self, hl_line: usize, col_start: usize, col_end: usize, scroll: usize) {
        if hl_line < scroll {
            return;
        }
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        let gutter_w = self.gutter_width();
        let avail = w.saturating_sub(gutter_w) as usize;
        let mut vis_row: usize = 0;
        for li in scroll..hl_line {
            vis_row += self.wrapped_line_rows(li, avail);
        }
        if vis_row >= h as usize {
            return;
        }
        let app = crate::app_palette::app_palette();
        let bg = app.editor.highlight_match.to_style().bg;
        let x_start = gutter_w + col_start as u16;
        let x_end = gutter_w + (col_end as u16).min(w.saturating_sub(gutter_w));
        let y = vis_row as u16;
        for x in x_start..x_end {
            self.state.buffer_mut().cell_mut(x, y).style.bg = bg;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::editor::EditorView;
    use txv_core::prelude::*;

    #[test]
    fn wide_char_positions_correct() {
        // "A✅B" — ✅ is width 2, so B should be at visual column 4
        let mut view = EditorView::from_text("A✅B");
        view.editor.options.number = false;
        view.set_bounds(Rect::new(0, 0, 20, 1));
        view.draw();
        let buf = view.buffer();
        assert_eq!(buf.cell(0, 0).ch, 'A');
        assert_eq!(buf.cell(1, 0).ch, '✅');
        assert_eq!(buf.cell(3, 0).ch, 'B');
    }

    #[test]
    fn matchparen_highlights_matching_bracket() {
        // Cursor on '(' at col 4 → matching ')' at col 7 should be highlighted.
        let mut view = EditorView::from_text("foo(bar)");
        view.editor.options.number = false;
        view.editor.options.matchparen = true;
        view.editor.cursor_col = 3; // on '('
        view.set_bounds(Rect::new(0, 0, 20, 1));
        view.draw();
        let buf = view.buffer();
        // The matching ')' at col 7 should have bold attrs (matchparen style).
        let cell = buf.cell(7, 0);
        assert_eq!(cell.ch, ')');
        assert!(cell.style.attrs.bold, "matching paren should be bold");
    }

    #[test]
    fn rainbow_brackets_colors_by_depth() {
        let result = rainbow_brackets("a(b(c))");
        // '(' at 1 depth 0, '(' at 3 depth 1, ')' at 5 depth 1, ')' at 6 depth 0
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].0, 1); // first '('
        assert_eq!(result[1].0, 3); // second '('
        assert_ne!(result[0].1, result[1].1); // different colors
        assert_eq!(result[0].1, result[3].1); // matching depth = same color
    }

    #[test]
    fn indent_guides_drawn_at_tab_stops() {
        let mut view = EditorView::from_text("        hello");
        view.editor.options.number = false;
        view.editor.options.guides = true;
        view.set_bounds(Rect::new(0, 0, 20, 1));
        view.draw();
        let buf = view.buffer();
        // 8 spaces = 2 indent levels at tab_width=4, guides at col 4.
        assert_eq!(buf.cell(4, 0).ch, '\u{2502}');
    }
}
