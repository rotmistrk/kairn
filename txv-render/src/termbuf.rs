//! TermBuf — VTE-driven virtual terminal emulator that renders to a Surface.

use txv_core::cell::{Color, Style};
use txv_core::surface::Surface;
use unicode_width::UnicodeWidthChar;

/// Virtual terminal buffer backed by VTE parser.
pub struct TermBuf {
    cols: u16,
    rows: u16,
    cells: Vec<Vec<TCell>>,
    cursor_x: u16,
    cursor_y: u16,
    cursor_visible: bool,
    style: Style,
    saved_cursor: (u16, u16),
    scroll_top: u16,
    scroll_bottom: u16,
    parser: vte::Parser,
}

#[derive(Clone)]
struct TCell {
    ch: char,
    style: Style,
    #[allow(dead_code)]
    width: u8,
}

impl Default for TCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
            width: 1,
        }
    }
}

impl TermBuf {
    pub fn new(cols: u16, rows: u16) -> Self {
        let cells = vec![vec![TCell::default(); cols as usize]; rows as usize];
        Self {
            cols,
            rows,
            cells,
            cursor_x: 0,
            cursor_y: 0,
            cursor_visible: true,
            style: Style::default(),
            saved_cursor: (0, 0),
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            parser: vte::Parser::new(),
        }
    }

    /// Feed bytes into the terminal emulator.
    pub fn process(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            let mut performer = Performer {
                cols: self.cols,
                rows: self.rows,
                cells: &mut self.cells,
                cursor_x: &mut self.cursor_x,
                cursor_y: &mut self.cursor_y,
                cursor_visible: &mut self.cursor_visible,
                style: &mut self.style,
                saved_cursor: &mut self.saved_cursor,
                scroll_top: &mut self.scroll_top,
                scroll_bottom: &mut self.scroll_bottom,
            };
            self.parser.advance(&mut performer, byte);
        }
    }

    /// Resize the terminal buffer.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let mut new_cells = vec![vec![TCell::default(); cols as usize]; rows as usize];
        let copy_rows = self.rows.min(rows) as usize;
        let copy_cols = self.cols.min(cols) as usize;
        for (y, new_row) in new_cells.iter_mut().enumerate().take(copy_rows) {
            new_row[..copy_cols].clone_from_slice(&self.cells[y][..copy_cols]);
        }
        self.cells = new_cells;
        self.cols = cols;
        self.rows = rows;
        self.scroll_bottom = rows.saturating_sub(1);
        self.cursor_x = self.cursor_x.min(cols.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(rows.saturating_sub(1));
    }

    /// Render terminal content to a Surface.
    pub fn render_to(&self, surface: &mut Surface) {
        let h = self.rows.min(surface.height());
        let w = self.cols.min(surface.width());
        for y in 0..h {
            for x in 0..w {
                let tc = &self.cells[y as usize][x as usize];
                surface.put(x, y, tc.ch, tc.style);
            }
        }
    }

    /// Current cursor position.
    pub fn cursor(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }

    /// Whether the cursor is visible.
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }
}

/// VTE Performer that mutates TermBuf state.
struct Performer<'a> {
    cols: u16,
    rows: u16,
    cells: &'a mut Vec<Vec<TCell>>,
    cursor_x: &'a mut u16,
    cursor_y: &'a mut u16,
    cursor_visible: &'a mut bool,
    style: &'a mut Style,
    saved_cursor: &'a mut (u16, u16),
    scroll_top: &'a mut u16,
    scroll_bottom: &'a mut u16,
}

impl Performer<'_> {
    fn put_char(&mut self, c: char) {
        let w = c.width().unwrap_or(0);
        if w == 0 {
            return;
        }
        if *self.cursor_x + (w as u16) > self.cols {
            self.newline();
            *self.cursor_x = 0;
        }
        let x = *self.cursor_x as usize;
        let y = *self.cursor_y as usize;
        if y < self.rows as usize && x < self.cols as usize {
            self.cells[y][x] = TCell {
                ch: c,
                style: *self.style,
                width: w as u8,
            };
            // Fill continuation cells for wide chars
            for i in 1..w {
                if x + i < self.cols as usize {
                    self.cells[y][x + i] = TCell {
                        ch: ' ',
                        style: *self.style,
                        width: 0,
                    };
                }
            }
        }
        *self.cursor_x += w as u16;
    }

    fn newline(&mut self) {
        if *self.cursor_y >= *self.scroll_bottom {
            self.scroll_up();
        } else {
            *self.cursor_y += 1;
        }
    }

    fn scroll_up(&mut self) {
        let top = *self.scroll_top as usize;
        let bot = *self.scroll_bottom as usize;
        if top < bot && bot < self.cells.len() {
            self.cells[top..=bot].rotate_left(1);
            let cols = self.cols as usize;
            self.cells[bot] = vec![TCell::default(); cols];
        }
    }

    fn scroll_down(&mut self) {
        let top = *self.scroll_top as usize;
        let bot = *self.scroll_bottom as usize;
        if top < bot && bot < self.cells.len() {
            self.cells[top..=bot].rotate_right(1);
            let cols = self.cols as usize;
            self.cells[top] = vec![TCell::default(); cols];
        }
    }

    fn erase_line(&mut self, mode: u16) {
        let y = *self.cursor_y as usize;
        if y >= self.rows as usize {
            return;
        }
        let (start, end) = match mode {
            0 => (*self.cursor_x as usize, self.cols as usize),
            1 => (0, (*self.cursor_x as usize) + 1),
            2 => (0, self.cols as usize),
            _ => return,
        };
        for x in start..end.min(self.cols as usize) {
            self.cells[y][x] = TCell::default();
        }
    }

    fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                // Cursor to end
                self.erase_line(0);
                for y in (*self.cursor_y as usize + 1)..self.rows as usize {
                    for x in 0..self.cols as usize {
                        self.cells[y][x] = TCell::default();
                    }
                }
            }
            1 => {
                // Start to cursor
                self.erase_line(1);
                for y in 0..*self.cursor_y as usize {
                    for x in 0..self.cols as usize {
                        self.cells[y][x] = TCell::default();
                    }
                }
            }
            2 | 3 => {
                // Entire screen
                for row in self.cells.iter_mut() {
                    for cell in row.iter_mut() {
                        *cell = TCell::default();
                    }
                }
            }
            _ => {}
        }
    }

    fn set_sgr(&mut self, params: &[u16]) {
        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => *self.style = Style::default(),
                1 => self.style.attrs.bold = true,
                2 => self.style.attrs.dim = true,
                3 => self.style.attrs.italic = true,
                4 => self.style.attrs.underline = true,
                7 => self.style.attrs.reverse = true,
                22 => {
                    self.style.attrs.bold = false;
                    self.style.attrs.dim = false;
                }
                23 => self.style.attrs.italic = false,
                24 => self.style.attrs.underline = false,
                27 => self.style.attrs.reverse = false,
                30..=37 => {
                    self.style.fg = Color::Ansi((params[i] - 30) as u8);
                }
                38 => {
                    i += 1;
                    if i < params.len() && params[i] == 5 {
                        i += 1;
                        if i < params.len() {
                            self.style.fg = Color::Palette(params[i] as u8);
                        }
                    } else if i < params.len() && params[i] == 2 && i + 3 < params.len() {
                        self.style.fg = Color::Rgb(params[i + 1] as u8, params[i + 2] as u8, params[i + 3] as u8);
                        i += 3;
                    }
                }
                39 => self.style.fg = Color::Reset,
                40..=47 => {
                    self.style.bg = Color::Ansi((params[i] - 40) as u8);
                }
                48 => {
                    i += 1;
                    if i < params.len() && params[i] == 5 {
                        i += 1;
                        if i < params.len() {
                            self.style.bg = Color::Palette(params[i] as u8);
                        }
                    } else if i < params.len() && params[i] == 2 && i + 3 < params.len() {
                        self.style.bg = Color::Rgb(params[i + 1] as u8, params[i + 2] as u8, params[i + 3] as u8);
                        i += 3;
                    }
                }
                49 => self.style.bg = Color::Reset,
                90..=97 => {
                    self.style.fg = Color::Ansi((params[i] - 90 + 8) as u8);
                }
                100..=107 => {
                    self.style.bg = Color::Ansi((params[i] - 100 + 8) as u8);
                }
                _ => {}
            }
            i += 1;
        }
    }
}

impl vte::Perform for Performer<'_> {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => *self.cursor_x = 0,
            b'\x08' => {
                *self.cursor_x = self.cursor_x.saturating_sub(1);
            }
            b'\t' => {
                let next_tab = ((*self.cursor_x / 8) + 1) * 8;
                *self.cursor_x = next_tab.min(self.cols.saturating_sub(1));
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &vte::Params, intermediates: &[u8], _ignore: bool, action: char) {
        let ps: Vec<u16> = params.iter().map(|p| p[0]).collect();
        let p1 = ps.first().copied().unwrap_or(0);

        match (action, intermediates) {
            ('A', []) => {
                // Cursor Up
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_y = self.cursor_y.saturating_sub(n);
            }
            ('B', []) => {
                // Cursor Down
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_y = (*self.cursor_y + n).min(self.rows.saturating_sub(1));
            }
            ('C', []) => {
                // Cursor Forward
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_x = (*self.cursor_x + n).min(self.cols.saturating_sub(1));
            }
            ('D', []) => {
                // Cursor Back
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_x = self.cursor_x.saturating_sub(n);
            }
            ('H' | 'f', []) => {
                // Cursor Position
                let row = if p1 == 0 {
                    1
                } else {
                    p1
                };
                let col = ps.get(1).copied().unwrap_or(1).max(1);
                *self.cursor_y = (row - 1).min(self.rows.saturating_sub(1));
                *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
            }
            ('G', []) => {
                // Cursor Character Absolute
                let col = if p1 == 0 {
                    1
                } else {
                    p1
                };
                *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
            }
            ('J', []) => {
                self.erase_display(p1);
            }
            ('K', []) => {
                self.erase_line(p1);
            }
            ('L', []) => {
                // Insert Lines
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let bot = *self.scroll_bottom as usize;
                for _ in 0..n {
                    if y <= bot && bot < self.cells.len() {
                        self.cells.remove(bot);
                        self.cells.insert(y, vec![TCell::default(); self.cols as usize]);
                    }
                }
            }
            ('M', []) => {
                // Delete Lines
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let bot = *self.scroll_bottom as usize;
                for _ in 0..n {
                    if y <= bot && bot < self.cells.len() {
                        self.cells.remove(y);
                        self.cells.insert(bot, vec![TCell::default(); self.cols as usize]);
                    }
                }
            }
            ('S', []) => {
                // Scroll Up
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                for _ in 0..n {
                    self.scroll_up();
                }
            }
            ('T', []) => {
                // Scroll Down
                let n = if p1 == 0 {
                    1
                } else {
                    p1
                };
                for _ in 0..n {
                    self.scroll_down();
                }
            }
            ('m', []) => {
                // SGR
                if ps.is_empty() {
                    self.set_sgr(&[0]);
                } else {
                    self.set_sgr(&ps);
                }
            }
            ('r', []) => {
                // Set Scrolling Region
                let top = if p1 == 0 {
                    1
                } else {
                    p1
                };
                let bot = ps.get(1).copied().unwrap_or(self.rows).min(self.rows);
                *self.scroll_top = top.saturating_sub(1);
                *self.scroll_bottom = bot.saturating_sub(1);
                *self.cursor_x = 0;
                *self.cursor_y = 0;
            }
            ('h', [b'?']) => {
                // DEC Private Mode Set
                if p1 == 25 {
                    *self.cursor_visible = true;
                }
            }
            ('l', [b'?']) => {
                // DEC Private Mode Reset
                if p1 == 25 {
                    *self.cursor_visible = false;
                }
            }
            ('s', []) => {
                // Save cursor
                *self.saved_cursor = (*self.cursor_x, *self.cursor_y);
            }
            ('u', []) => {
                // Restore cursor
                *self.cursor_x = self.saved_cursor.0;
                *self.cursor_y = self.saved_cursor.1;
            }
            ('P', []) => {
                // Delete Characters
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let x = *self.cursor_x as usize;
                if y < self.rows as usize {
                    let row = &mut self.cells[y];
                    let end = (x + n).min(row.len());
                    row.drain(x..end);
                    row.resize(self.cols as usize, TCell::default());
                }
            }
            ('@', []) => {
                // Insert Characters
                let n = if p1 == 0 {
                    1
                } else {
                    p1 as usize
                };
                let y = *self.cursor_y as usize;
                let x = *self.cursor_x as usize;
                if y < self.rows as usize {
                    let row = &mut self.cells[y];
                    for _ in 0..n {
                        if x < row.len() {
                            row.insert(x, TCell::default());
                        }
                    }
                    row.truncate(self.cols as usize);
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates) {
            (b'7', []) => {
                *self.saved_cursor = (*self.cursor_x, *self.cursor_y);
            }
            (b'8', []) => {
                *self.cursor_x = self.saved_cursor.0;
                *self.cursor_y = self.saved_cursor.1;
            }
            (b'D', []) => self.newline(),
            (b'M', []) => {
                // Reverse Index
                if *self.cursor_y <= *self.scroll_top {
                    self.scroll_down();
                } else {
                    *self.cursor_y -= 1;
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_print() {
        let mut tb = TermBuf::new(80, 24);
        tb.process(b"Hello");
        assert_eq!(tb.cells[0][0].ch, 'H');
        assert_eq!(tb.cells[0][4].ch, 'o');
        assert_eq!(tb.cursor(), (5, 0));
    }

    #[test]
    fn newline_and_cr() {
        let mut tb = TermBuf::new(80, 24);
        tb.process(b"A\r\nB");
        assert_eq!(tb.cells[0][0].ch, 'A');
        assert_eq!(tb.cells[1][0].ch, 'B');
    }

    #[test]
    fn cursor_movement() {
        let mut tb = TermBuf::new(80, 24);
        tb.process(b"\x1b[5;10H");
        assert_eq!(tb.cursor(), (9, 4));
    }

    #[test]
    fn erase_line() {
        let mut tb = TermBuf::new(80, 24);
        tb.process(b"ABCDEF\x1b[4G\x1b[K");
        assert_eq!(tb.cells[0][0].ch, 'A');
        assert_eq!(tb.cells[0][1].ch, 'B');
        assert_eq!(tb.cells[0][2].ch, 'C');
        assert_eq!(tb.cells[0][3].ch, ' ');
    }

    #[test]
    fn sgr_colors() {
        let mut tb = TermBuf::new(80, 24);
        tb.process(b"\x1b[31mR\x1b[0m");
        assert_eq!(tb.cells[0][0].ch, 'R');
        assert_eq!(tb.cells[0][0].style.fg, Color::Ansi(1));
    }

    #[test]
    fn scroll_on_overflow() {
        let mut tb = TermBuf::new(80, 3);
        tb.process(b"A\r\nB\r\nC\r\nD");
        // After scrolling, row 0 should be B, row 1 C, row 2 D
        assert_eq!(tb.cells[0][0].ch, 'B');
        assert_eq!(tb.cells[1][0].ch, 'C');
        assert_eq!(tb.cells[2][0].ch, 'D');
    }

    #[test]
    fn render_to_surface() {
        let mut tb = TermBuf::new(10, 5);
        tb.process(b"Hi");
        let mut surface = Surface::new(10, 5);
        tb.render_to(&mut surface);
        assert_eq!(surface.cell(0, 0).ch, 'H');
        assert_eq!(surface.cell(1, 0).ch, 'i');
    }

    #[test]
    fn resize_preserves_content() {
        let mut tb = TermBuf::new(80, 24);
        tb.process(b"Hello");
        tb.resize(40, 12);
        assert_eq!(tb.cells[0][0].ch, 'H');
        assert_eq!(tb.cols, 40);
        assert_eq!(tb.rows, 12);
    }

    #[test]
    fn cursor_visibility() {
        let mut tb = TermBuf::new(80, 24);
        assert!(tb.cursor_visible());
        tb.process(b"\x1b[?25l");
        assert!(!tb.cursor_visible());
        tb.process(b"\x1b[?25h");
        assert!(tb.cursor_visible());
    }
}
