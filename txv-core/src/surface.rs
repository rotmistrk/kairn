//! Surface — abstract drawing target backed by a cell grid.

use crate::cell::{Cell, Style};

/// Owned cell grid.
pub struct Surface {
    cells: Vec<Cell>,
    width: u16,
    height: u16,
}

impl Surface {
    pub fn new(w: u16, h: u16) -> Self {
        let len = (w as usize) * (h as usize);
        Self {
            cells: vec![Cell::default(); len],
            width: w,
            height: h,
        }
    }

    pub fn width(&self) -> u16 {
        self.width
    }

    pub fn height(&self) -> u16 {
        self.height
    }

    pub fn cell(&self, x: u16, y: u16) -> &Cell {
        &self.cells[self.idx(x, y)]
    }

    pub fn cell_mut(&mut self, x: u16, y: u16) -> &mut Cell {
        let i = self.idx(x, y);
        &mut self.cells[i]
    }

    pub fn put(&mut self, x: u16, y: u16, ch: char, style: Style) {
        if x < self.width && y < self.height {
            let i = self.idx(x, y);
            self.cells[i] = Cell { ch, style, width: 1 };
        }
    }

    pub fn print(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut col = x;
        for ch in text.chars() {
            if col >= self.width {
                break;
            }
            self.put(col, y, ch, style);
            col = col.saturating_add(1);
        }
    }

    /// Print text at (x, y) and fill remaining width with spaces in the same style.
    /// TXV model: every line write covers the full width. No stale content.
    pub fn print_line(&mut self, x: u16, y: u16, text: &str, width: u16, style: Style) {
        let mut col = x;
        let end = x.saturating_add(width).min(self.width);
        for ch in text.chars() {
            if col >= end { break; }
            self.put(col, y, ch, style);
            col = col.saturating_add(1);
        }
        while col < end {
            self.put(col, y, ' ', style);
            col += 1;
        }
    }

    /// Print styled spans at (x, y) and fill remaining width with spaces.
    /// TXV model: every line write covers the full width. No stale content.
    pub fn print_spans_line(
        &mut self,
        x: u16,
        y: u16,
        spans: &[(&str, Style)],
        width: u16,
        fill_style: Style,
    ) {
        let mut col = x;
        let end = x.saturating_add(width).min(self.width);
        for &(text, style) in spans {
            for ch in text.chars() {
                if col >= end { break; }
                self.put(col, y, ch, style);
                col = col.saturating_add(1);
            }
        }
        while col < end {
            self.put(col, y, ' ', fill_style);
            col += 1;
        }
    }

    pub fn fill(&mut self, ch: char, style: Style) {
        for cell in &mut self.cells {
            *cell = Cell { ch, style, width: 1 };
        }
    }

    pub fn hline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style) {
        for col in x..x.saturating_add(len).min(self.width) {
            self.put(col, y, ch, style);
        }
    }

    pub fn vline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style) {
        for row in y..y.saturating_add(len).min(self.height) {
            self.put(x, row, ch, style);
        }
    }

    pub fn sub(&mut self, x: u16, y: u16, w: u16, h: u16) -> SubSurface<'_> {
        let clamped_w = w.min(self.width.saturating_sub(x));
        let clamped_h = h.min(self.height.saturating_sub(y));
        SubSurface {
            surface: self,
            ox: x,
            oy: y,
            w: clamped_w,
            h: clamped_h,
        }
    }

    fn idx(&self, x: u16, y: u16) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }
}

/// A borrowed rectangular region of a Surface.
pub struct SubSurface<'a> {
    surface: &'a mut Surface,
    ox: u16,
    oy: u16,
    w: u16,
    h: u16,
}

impl SubSurface<'_> {
    pub fn width(&self) -> u16 {
        self.w
    }

    pub fn height(&self) -> u16 {
        self.h
    }

    pub fn put(&mut self, x: u16, y: u16, ch: char, style: Style) {
        if x < self.w && y < self.h {
            self.surface
                .put(self.ox.saturating_add(x), self.oy.saturating_add(y), ch, style);
        }
    }

    pub fn print(&mut self, x: u16, y: u16, text: &str, style: Style) {
        let mut col = x;
        for ch in text.chars() {
            if col >= self.w {
                break;
            }
            self.put(col, y, ch, style);
            col = col.saturating_add(1);
        }
    }

    /// Print text at (x, y) and fill remaining width with spaces.
    pub fn print_line(&mut self, x: u16, y: u16, text: &str, width: u16, style: Style) {
        let mut col = x;
        let end = x.saturating_add(width).min(self.w);
        for ch in text.chars() {
            if col >= end { break; }
            self.put(col, y, ch, style);
            col = col.saturating_add(1);
        }
        while col < end {
            self.put(col, y, ' ', style);
            col += 1;
        }
    }

    /// Print styled spans at (x, y) and fill remaining width with spaces.
    pub fn print_spans_line(
        &mut self,
        x: u16,
        y: u16,
        spans: &[(&str, Style)],
        width: u16,
        fill_style: Style,
    ) {
        let mut col = x;
        let end = x.saturating_add(width).min(self.w);
        for &(text, style) in spans {
            for ch in text.chars() {
                if col >= end { break; }
                self.put(col, y, ch, style);
                col = col.saturating_add(1);
            }
        }
        while col < end {
            self.put(col, y, ' ', fill_style);
            col += 1;
        }
    }

    pub fn fill(&mut self, ch: char, style: Style) {
        for row in 0..self.h {
            for col in 0..self.w {
                self.put(col, row, ch, style);
            }
        }
    }

    pub fn hline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style) {
        for col in x..x.saturating_add(len).min(self.w) {
            self.put(col, y, ch, style);
        }
    }

    pub fn vline(&mut self, x: u16, y: u16, len: u16, ch: char, style: Style) {
        for row in y..y.saturating_add(len).min(self.h) {
            self.put(x, row, ch, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::Style;

    #[test]
    fn surface_put_and_cell() {
        let mut s = Surface::new(10, 5);
        let style = Style::default();
        s.put(3, 2, 'X', style);
        assert_eq!(s.cell(3, 2).ch, 'X');
        assert_eq!(s.cell(0, 0).ch, ' ');
    }

    #[test]
    fn surface_print() {
        let mut s = Surface::new(10, 1);
        s.print(0, 0, "Hello", Style::default());
        assert_eq!(s.cell(0, 0).ch, 'H');
        assert_eq!(s.cell(4, 0).ch, 'o');
    }

    #[test]
    fn subsurface_clips() {
        let mut s = Surface::new(10, 10);
        {
            let mut sub = s.sub(5, 5, 3, 3);
            sub.put(0, 0, 'A', Style::default());
            sub.put(5, 5, 'B', Style::default()); // out of bounds, ignored
        }
        assert_eq!(s.cell(5, 5).ch, 'A');
        assert_eq!(s.cell(0, 0).ch, ' ');
    }
}

/// Display width of a character (1 for normal, 2 for wide/CJK).
pub fn display_char_width(ch: char) -> u16 {
    let cp = ch as u32;
    if (0x1100..=0x115F).contains(&cp)
        || (0x2E80..=0x303E).contains(&cp)
        || (0x3041..=0x33BF).contains(&cp)
        || (0x3400..=0x4DBF).contains(&cp)
        || (0x4E00..=0x9FFF).contains(&cp)
        || (0xAC00..=0xD7AF).contains(&cp)
        || (0xF900..=0xFAFF).contains(&cp)
        || (0xFE30..=0xFE6F).contains(&cp)
        || (0xFF01..=0xFF60).contains(&cp)
        || (0xFFE0..=0xFFE6).contains(&cp)
        || (0x20000..=0x2FFFD).contains(&cp)
        || (0x30000..=0x3FFFD).contains(&cp)
        || (0x2600..=0x27BF).contains(&cp)  // Misc symbols (✅, etc.)
        || (0x1F300..=0x1F9FF).contains(&cp) // Emoji
    {
        2
    } else {
        1
    }
}

/// Iterate characters with their visual column positions.
/// Handles wide chars (2 cells) and tabs (tab_width cells).
/// Apps use this instead of manual col tracking.
///
/// Returns: Vec of (visual_col, char, char_display_width).
///
/// # Example
///
/// ```
/// use txv_core::surface::{visual_positions, display_width};
/// # use txv_core::prelude::*;
/// # let mut surface = Surface::new(40, 1);
/// # let x = 0u16;
/// # let y = 0u16;
/// # let style = Style::default();
/// for (col, ch, _width) in visual_positions("hello ✅ world", 4) {
///     surface.put(x + col, y, ch, style);
/// }
/// // Padding starts at x + display_width("hello ✅ world", 4)
/// ```
pub fn visual_positions(text: &str, tab_width: usize) -> Vec<(u16, char, u16)> {
    let mut col: u16 = 0;
    let mut result = Vec::new();
    for ch in text.chars() {
        let w = if ch == '\t' { tab_width as u16 } else { display_char_width(ch) };
        result.push((col, ch, w));
        col += w;
    }
    result
}

/// Total display width of a string (accounts for wide chars and tabs).
pub fn display_width(text: &str, tab_width: usize) -> u16 {
    let mut col: u16 = 0;
    for ch in text.chars() {
        col += if ch == '\t' { tab_width as u16 } else { display_char_width(ch) };
    }
    col
}
