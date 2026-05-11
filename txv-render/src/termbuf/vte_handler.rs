//! VTE Performer implementation for TermBuf.

use txv_core::cell::{Color, Style};
use unicode_width::UnicodeWidthChar;

use super::scrollback::Scrollback;
use super::TCell;

/// VTE Performer that mutates TermBuf state.
pub(super) struct Performer<'a> {
    pub cols: u16,
    pub rows: u16,
    pub cells: &'a mut Vec<Vec<TCell>>,
    pub cursor_x: &'a mut u16,
    pub cursor_y: &'a mut u16,
    pub cursor_visible: &'a mut bool,
    pub style: &'a mut Style,
    pub saved_cursor: &'a mut (u16, u16),
    pub scroll_top: &'a mut u16,
    pub scroll_bottom: &'a mut u16,
    pub responses: &'a mut Vec<Vec<u8>>,
    pub swallow_flag: &'a mut bool,
    pub osc_title: &'a mut Option<String>,
    pub scrollback: &'a mut Scrollback,
}

impl Performer<'_> {
    pub(super) fn put_char(&mut self, c: char) {
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

    pub(super) fn newline(&mut self) {
        if *self.cursor_y >= *self.scroll_bottom {
            self.scroll_up();
        } else {
            *self.cursor_y += 1;
        }
    }

    pub(super) fn scroll_up(&mut self) {
        let top = *self.scroll_top as usize;
        let bot = *self.scroll_bottom as usize;
        if top < bot && bot < self.cells.len() {
            // Save the line being pushed off (only when scrolling the full screen)
            if top == 0 {
                self.scrollback.push(self.cells[0].clone());
            }
            self.cells[top..=bot].rotate_left(1);
            self.cells[bot] = vec![TCell::default(); self.cols as usize];
        }
    }

    pub(super) fn scroll_down(&mut self) {
        let top = *self.scroll_top as usize;
        let bot = *self.scroll_bottom as usize;
        if top < bot && bot < self.cells.len() {
            self.cells[top..=bot].rotate_right(1);
            self.cells[top] = vec![TCell::default(); self.cols as usize];
        }
    }

    pub(super) fn erase_line(&mut self, mode: u16) {
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

    pub(super) fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                self.erase_line(0);
                for y in (*self.cursor_y as usize + 1)..self.rows as usize {
                    for x in 0..self.cols as usize {
                        self.cells[y][x] = TCell::default();
                    }
                }
            }
            1 => {
                self.erase_line(1);
                for y in 0..*self.cursor_y as usize {
                    for x in 0..self.cols as usize {
                        self.cells[y][x] = TCell::default();
                    }
                }
            }
            2 | 3 => {
                for row in self.cells.iter_mut() {
                    for cell in row.iter_mut() {
                        *cell = TCell::default();
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn set_sgr(&mut self, params: &[u16]) {
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
                30..=37 => self.style.fg = Color::Ansi((params[i] - 30) as u8),
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
                40..=47 => self.style.bg = Color::Ansi((params[i] - 40) as u8),
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
                90..=97 => self.style.fg = Color::Ansi((params[i] - 90 + 8) as u8),
                100..=107 => self.style.bg = Color::Ansi((params[i] - 100 + 8) as u8),
                _ => {}
            }
            i += 1;
        }
    }
}
