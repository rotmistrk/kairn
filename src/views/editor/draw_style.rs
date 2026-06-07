//! Style helpers for editor draw — computes per-character styles.

use txv_core::prelude::{Color, Style};

/// Compute the effective style for a character at `byte_pos`, applying
/// visual selection and search highlights.
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
            return Style::new(base.fg(), visual_bg).with_attrs(base.attrs());
        }
    } else if let Some(is_current) = highlight.and_then(|h| h.match_at(byte_pos)) {
        if is_current {
            return hl_match;
        }
        return Style::new(base.fg(), hl_other_bg).with_attrs(base.attrs());
    }
    base
}

/// Apply matchparen and rainbow bracket highlights to a character style.
pub(super) fn bracket_highlight(
    base: Style,
    line_idx: usize,
    char_idx: usize,
    matchparen_pos: Option<(usize, usize)>,
    matchparen_style: &Style,
    rainbow_map: &[(usize, Color)],
) -> Style {
    if matchparen_pos == Some((line_idx, char_idx)) {
        let fg = if matchparen_style.fg() != Color::Reset {
            matchparen_style.fg()
        } else {
            base.fg()
        };
        let bg = if matchparen_style.bg() != Color::Reset {
            matchparen_style.bg()
        } else {
            base.bg()
        };
        Style::new(fg, bg).with_attrs(matchparen_style.attrs())
    } else if let Some(&(_, color)) = rainbow_map.iter().find(|(col, _)| *col == char_idx) {
        base.with_fg(color)
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
        buf.put(text_x + g as u16, vy, '\u{250A}', style);
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
    rainbow_brackets_with_depth(line, 0).0
}

/// Compute rainbow bracket colors for a line starting at a given depth.
/// Returns (vec of (char_index, color), ending depth).
pub(super) fn rainbow_brackets_with_depth(line: &str, mut depth: usize) -> (Vec<(usize, Color)>, usize) {
    let mut result = Vec::new();
    for (idx, ch) in line.chars().enumerate() {
        match ch {
            '(' | '[' | '{' => {
                result.push((idx, RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]));
                depth += 1;
            }
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                result.push((idx, RAINBOW_COLORS[depth % RAINBOW_COLORS.len()]));
            }
            _ => {}
        }
    }
    (result, depth)
}

/// Compute the bracket depth at a given line by scanning all lines before it.
pub(super) fn bracket_depth_at_line(buf: &crate::buffer::piece_table::PieceTable, line: usize) -> usize {
    let mut depth: usize = 0;
    for i in 0..line {
        let text = buf.line(i).unwrap_or_default();
        for ch in text.chars() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                _ => {}
            }
        }
    }
    depth
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
            let prompt_style = txv_core::palette::palette().style(txv_core::palette::StyleId::StatusBar);
            let prefix = if self.editor.mode == crate::editor::keymap::EditorMode::Search {
                "/"
            } else {
                ":"
            };
            let prompt_text = format!("{}{}", prefix, self.editor.command_buf);
            self.state
                .buffer_mut()
                .print_line(0, prompt_y, &prompt_text, w, prompt_style);
            // Draw cursor on prompt
            if self.state.is_focused() {
                let cx = prompt_text.len() as u16;
                if cx < w {
                    let cursor_style = txv_core::palette::palette().style(txv_core::palette::StyleId::InputCursor);
                    self.state.buffer_mut().put(cx, prompt_y, ' ', cursor_style);
                }
            }
        }
    }
}

/// Paint highlight background on a single row (for gs target line).
pub(super) fn paint_line_bg(buf: &mut txv_core::buffer::Buffer, y: u16, from_x: u16, to_x: u16) {
    let bg = txv_core::palette::palette()
        .style(txv_core::palette::StyleId::SearchMatch)
        .bg();
    if bg == Color::Reset {
        return;
    }
    let bw = buf.width() as usize;
    let cells = buf.cells_mut();
    let base = y as usize * bw;
    for x in (from_x as usize)..(to_x as usize) {
        if let Some(c) = cells.get_mut(base + x) {
            c.style_mut().set_bg(bg);
        }
    }
}

impl super::EditorView {
    pub(super) fn ephemeral_fill(&self, line_idx: usize, p: &super::draw::DrawParams) -> Style {
        if self.editor.ephemeral.ranges.iter().any(|r| r.covers_line(line_idx)) {
            Style::default().with_bg(p.ephemeral_bg)
        } else {
            Style::default()
        }
    }

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
        let bg = app.editor().highlight_match().bg();
        let x_start = gutter_w + col_start as u16;
        let x_end = gutter_w + (col_end as u16).min(w.saturating_sub(gutter_w));
        let y = vis_row as u16;
        for x in x_start..x_end {
            self.state.buffer_mut().cell_mut(x, y).style_mut().set_bg(bg);
        }
    }
}

#[cfg(test)]
#[path = "draw_style_tests.rs"]
mod tests;
