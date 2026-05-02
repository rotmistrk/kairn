// Screen — dual-buffer differential rendering.

use std::io::{self, Write};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    style::{
        Attribute, Color as CtColor, Print, SetAttribute, SetBackgroundColor, SetForegroundColor,
    },
    QueueableCommand,
};

use crate::cell::{
    detect_color_mode, rgb_to_ansi, rgb_to_palette, Attrs, Cell, Color, ColorMode, Style,
};
use crate::surface::Surface;

/// Full terminal screen with dual-buffer differential rendering.
pub struct Screen {
    width: u16,
    height: u16,
    current: Vec<Cell>,
    previous: Vec<Cell>,
    color_mode: ColorMode,
    cursor_pos: Option<(u16, u16)>,
}

impl Screen {
    /// Create a new screen. Detects color mode from environment.
    pub fn new(width: u16, height: u16) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            current: vec![Cell::default(); size],
            previous: vec![Cell::default(); size],
            color_mode: detect_color_mode(),
            cursor_pos: None,
        }
    }

    /// Create a screen with explicit color mode (for testing).
    pub fn with_color_mode(width: u16, height: u16, color_mode: ColorMode) -> Self {
        let size = width as usize * height as usize;
        Self {
            width,
            height,
            current: vec![Cell::default(); size],
            previous: vec![Cell::default(); size],
            color_mode,
            cursor_pos: None,
        }
    }

    /// Resize the screen. Clears both grids.
    pub fn resize(&mut self, width: u16, height: u16) {
        let size = width as usize * height as usize;
        self.width = width;
        self.height = height;
        self.current = vec![Cell::default(); size];
        self.previous = vec![Cell::default(); size];
    }

    /// Get a surface covering a rectangular region of the current grid.
    pub fn surface(&mut self, col: u16, row: u16, w: u16, h: u16) -> Surface<'_> {
        Surface::new(&mut self.current, self.width, col, row, w, h)
    }

    /// Get a surface covering the entire screen.
    pub fn full_surface(&mut self) -> Surface<'_> {
        let w = self.width;
        let h = self.height;
        Surface::new(&mut self.current, w, 0, 0, w, h)
    }

    /// Set cursor position (shown after flush). None = hidden.
    pub fn set_cursor(&mut self, pos: Option<(u16, u16)>) {
        self.cursor_pos = pos;
    }

    /// Flush changes to the terminal. Only emits diffs.
    pub fn flush(&mut self, out: &mut impl Write) -> io::Result<()> {
        let mut last_style: Option<Style> = None;
        let mut cursor_col: u16 = u16::MAX;
        let mut cursor_row: u16 = u16::MAX;

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = row as usize * self.width as usize + col as usize;
                let cur = &self.current[idx];
                let prev = &self.previous[idx];
                if cur == prev {
                    continue;
                }
                // Move cursor if not at expected position
                if cursor_col != col || cursor_row != row {
                    out.queue(MoveTo(col, row))?;
                }
                // Emit style if changed
                if last_style.as_ref() != Some(&cur.style) {
                    self.emit_style(out, &cur.style)?;
                    last_style = Some(cur.style);
                }
                // Skip continuation cells (width 0)
                if cur.width == 0 {
                    cursor_col = col + 1;
                    cursor_row = row;
                    continue;
                }
                out.queue(Print(cur.ch))?;
                cursor_col = col + cur.width as u16;
                cursor_row = row;
            }
        }

        // Copy current to previous
        self.previous.clone_from(&self.current);

        // Reset attributes
        out.queue(SetAttribute(Attribute::Reset))?;

        // Handle cursor
        match self.cursor_pos {
            Some((col, row)) => {
                out.queue(Show)?;
                out.queue(MoveTo(col, row))?;
            }
            None => {
                out.queue(Hide)?;
            }
        }

        out.flush()
    }

    /// Mark all cells as dirty (forces full redraw on next flush).
    pub fn force_redraw(&mut self) {
        for cell in &mut self.previous {
            // Set to a sentinel that won't match any real cell
            cell.ch = '\x01';
        }
    }

    /// Read a cell from the current grid (for testing).
    pub fn cell(&self, col: u16, row: u16) -> &Cell {
        let idx = row as usize * self.width as usize + col as usize;
        &self.current[idx]
    }

    /// Dump current grid as plain text (for test assertions).
    /// Skips continuation cells (width 0).
    pub fn to_text(&self) -> String {
        let mut result = String::new();
        for row in 0..self.height {
            for col in 0..self.width {
                let idx = row as usize * self.width as usize + col as usize;
                let cell = &self.current[idx];
                if cell.width == 0 {
                    continue;
                }
                result.push(cell.ch);
            }
            result.push('\n');
        }
        result
    }

    /// Screen width.
    pub fn width(&self) -> u16 {
        self.width
    }

    /// Screen height.
    pub fn height(&self) -> u16 {
        self.height
    }

    fn emit_style(&self, out: &mut impl Write, style: &Style) -> io::Result<()> {
        out.queue(SetAttribute(Attribute::Reset))?;
        out.queue(SetForegroundColor(self.convert_color(style.fg)))?;
        out.queue(SetBackgroundColor(self.convert_color(style.bg)))?;
        self.emit_attrs(out, &style.attrs)
    }

    fn emit_attrs(&self, out: &mut impl Write, attrs: &Attrs) -> io::Result<()> {
        if attrs.bold {
            out.queue(SetAttribute(Attribute::Bold))?;
        }
        if attrs.italic {
            out.queue(SetAttribute(Attribute::Italic))?;
        }
        if attrs.underline {
            out.queue(SetAttribute(Attribute::Underlined))?;
        }
        if attrs.reverse {
            out.queue(SetAttribute(Attribute::Reverse))?;
        }
        if attrs.dim {
            out.queue(SetAttribute(Attribute::Dim))?;
        }
        if attrs.strikethrough {
            out.queue(SetAttribute(Attribute::CrossedOut))?;
        }
        Ok(())
    }

    fn convert_color(&self, color: Color) -> CtColor {
        match color {
            Color::Reset => CtColor::Reset,
            Color::Ansi(n) => CtColor::AnsiValue(n),
            Color::Palette(n) => match self.color_mode {
                ColorMode::Ansi => CtColor::AnsiValue(n),
                _ => CtColor::AnsiValue(n),
            },
            Color::Rgb(r, g, b) => match self.color_mode {
                ColorMode::Rgb => CtColor::Rgb { r, g, b },
                ColorMode::Palette => CtColor::AnsiValue(rgb_to_palette(r, g, b)),
                ColorMode::Ansi => CtColor::AnsiValue(rgb_to_ansi(r, g, b)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_screen_default_cells() {
        let screen = Screen::with_color_mode(10, 5, ColorMode::Rgb);
        assert_eq!(screen.width(), 10);
        assert_eq!(screen.height(), 5);
        assert_eq!(screen.cell(0, 0).ch, ' ');
    }

    #[test]
    fn surface_writes_to_current() {
        let mut screen = Screen::with_color_mode(10, 3, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            s.put(0, 0, 'H', Style::default());
        }
        assert_eq!(screen.cell(0, 0).ch, 'H');
    }

    #[test]
    fn flush_no_changes_emits_minimal() {
        let mut screen = Screen::with_color_mode(5, 2, ColorMode::Rgb);
        let mut buf = Vec::new();
        screen.flush(&mut buf).ok();
        // Should only have reset + cursor hide, no cell data
        let s = String::from_utf8_lossy(&buf);
        assert!(!s.contains('H'));
    }

    #[test]
    fn flush_emits_changed_cell() {
        let mut screen = Screen::with_color_mode(10, 3, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            s.put(0, 0, 'X', Style::default());
        }
        let mut buf = Vec::new();
        screen.flush(&mut buf).ok();
        let s = String::from_utf8_lossy(&buf);
        assert!(s.contains('X'));
    }

    #[test]
    fn flush_second_time_no_changes() {
        let mut screen = Screen::with_color_mode(10, 3, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            s.put(0, 0, 'A', Style::default());
        }
        let mut buf1 = Vec::new();
        screen.flush(&mut buf1).ok();

        // Second flush with no changes
        let mut buf2 = Vec::new();
        screen.flush(&mut buf2).ok();
        // buf2 should be smaller (no cell data)
        assert!(buf2.len() < buf1.len());
    }

    #[test]
    fn resize_clears_grids() {
        let mut screen = Screen::with_color_mode(10, 5, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            s.put(0, 0, 'Z', Style::default());
        }
        screen.resize(20, 10);
        assert_eq!(screen.width(), 20);
        assert_eq!(screen.height(), 10);
        assert_eq!(screen.cell(0, 0).ch, ' ');
    }

    #[test]
    fn force_redraw_makes_all_dirty() {
        let mut screen = Screen::with_color_mode(5, 2, ColorMode::Rgb);
        // Flush once to sync grids
        let mut buf = Vec::new();
        screen.flush(&mut buf).ok();

        screen.force_redraw();
        let mut buf2 = Vec::new();
        screen.flush(&mut buf2).ok();
        // Should emit all cells since previous was invalidated
        assert!(buf2.len() > buf.len());
    }

    #[test]
    fn to_text_output() {
        let mut screen = Screen::with_color_mode(5, 2, ColorMode::Rgb);
        {
            let mut s = screen.full_surface();
            s.print(0, 0, "Hello", Style::default());
            s.print(0, 1, "World", Style::default());
        }
        let text = screen.to_text();
        assert_eq!(text, "Hello\nWorld\n");
    }

    #[test]
    fn cursor_position() {
        let mut screen = Screen::with_color_mode(10, 5, ColorMode::Rgb);
        screen.set_cursor(Some((3, 2)));
        let mut buf = Vec::new();
        screen.flush(&mut buf).ok();
        let s = String::from_utf8_lossy(&buf);
        // Should contain Show cursor sequence
        assert!(s.contains("\x1b[?25h"));
    }

    #[test]
    fn cursor_hidden() {
        let mut screen = Screen::with_color_mode(10, 5, ColorMode::Rgb);
        screen.set_cursor(None);
        let mut buf = Vec::new();
        screen.flush(&mut buf).ok();
        let s = String::from_utf8_lossy(&buf);
        // Should contain Hide cursor sequence
        assert!(s.contains("\x1b[?25l"));
    }

    #[test]
    fn sub_surface_from_screen() {
        let mut screen = Screen::with_color_mode(20, 10, ColorMode::Rgb);
        {
            let mut s = screen.surface(5, 5, 10, 5);
            s.put(0, 0, 'Q', Style::default());
        }
        assert_eq!(screen.cell(5, 5).ch, 'Q');
        assert_eq!(screen.cell(0, 0).ch, ' ');
    }
}
