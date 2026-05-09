//! VTE Performer implementation for TermBuf.

use txv_core::cell::{Color, Style};
use unicode_width::UnicodeWidthChar;

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
}

impl Performer<'_> {
    fn put_char(&mut self, c: char) {
        let w = c.width().unwrap_or(0);
        if w == 0 { return; }
        if *self.cursor_x + (w as u16) > self.cols {
            self.newline();
            *self.cursor_x = 0;
        }
        let x = *self.cursor_x as usize;
        let y = *self.cursor_y as usize;
        if y < self.rows as usize && x < self.cols as usize {
            self.cells[y][x] = TCell { ch: c, style: *self.style, width: w as u8 };
            for i in 1..w {
                if x + i < self.cols as usize {
                    self.cells[y][x + i] = TCell { ch: ' ', style: *self.style, width: 0 };
                }
            }
        }
        *self.cursor_x += w as u16;
    }

    fn newline(&mut self) {
        if *self.cursor_y >= *self.scroll_bottom { self.scroll_up(); } else { *self.cursor_y += 1; }
    }

    fn scroll_up(&mut self) {
        let top = *self.scroll_top as usize;
        let bot = *self.scroll_bottom as usize;
        if top < bot && bot < self.cells.len() {
            self.cells[top..=bot].rotate_left(1);
            self.cells[bot] = vec![TCell::default(); self.cols as usize];
        }
    }

    fn scroll_down(&mut self) {
        let top = *self.scroll_top as usize;
        let bot = *self.scroll_bottom as usize;
        if top < bot && bot < self.cells.len() {
            self.cells[top..=bot].rotate_right(1);
            self.cells[top] = vec![TCell::default(); self.cols as usize];
        }
    }

    fn erase_line(&mut self, mode: u16) {
        let y = *self.cursor_y as usize;
        if y >= self.rows as usize { return; }
        let (start, end) = match mode {
            0 => (*self.cursor_x as usize, self.cols as usize),
            1 => (0, (*self.cursor_x as usize) + 1),
            2 => (0, self.cols as usize),
            _ => return,
        };
        for x in start..end.min(self.cols as usize) { self.cells[y][x] = TCell::default(); }
    }

    fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                self.erase_line(0);
                for y in (*self.cursor_y as usize + 1)..self.rows as usize {
                    for x in 0..self.cols as usize { self.cells[y][x] = TCell::default(); }
                }
            }
            1 => {
                self.erase_line(1);
                for y in 0..*self.cursor_y as usize {
                    for x in 0..self.cols as usize { self.cells[y][x] = TCell::default(); }
                }
            }
            2 | 3 => {
                for row in self.cells.iter_mut() {
                    for cell in row.iter_mut() { *cell = TCell::default(); }
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
                22 => { self.style.attrs.bold = false; self.style.attrs.dim = false; }
                23 => self.style.attrs.italic = false,
                24 => self.style.attrs.underline = false,
                27 => self.style.attrs.reverse = false,
                30..=37 => self.style.fg = Color::Ansi((params[i] - 30) as u8),
                38 => {
                    i += 1;
                    if i < params.len() && params[i] == 5 {
                        i += 1;
                        if i < params.len() { self.style.fg = Color::Palette(params[i] as u8); }
                    } else if i < params.len() && params[i] == 2 && i + 3 < params.len() {
                        self.style.fg = Color::Rgb(params[i+1] as u8, params[i+2] as u8, params[i+3] as u8);
                        i += 3;
                    }
                }
                39 => self.style.fg = Color::Reset,
                40..=47 => self.style.bg = Color::Ansi((params[i] - 40) as u8),
                48 => {
                    i += 1;
                    if i < params.len() && params[i] == 5 {
                        i += 1;
                        if i < params.len() { self.style.bg = Color::Palette(params[i] as u8); }
                    } else if i < params.len() && params[i] == 2 && i + 3 < params.len() {
                        self.style.bg = Color::Rgb(params[i+1] as u8, params[i+2] as u8, params[i+3] as u8);
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

impl vte::Perform for Performer<'_> {
    fn print(&mut self, c: char) { self.put_char(c); }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => self.newline(),
            b'\r' => *self.cursor_x = 0,
            b'\x08' => { *self.cursor_x = self.cursor_x.saturating_sub(1); }
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
            ('A', []) => { let n = if p1 == 0 { 1 } else { p1 }; *self.cursor_y = self.cursor_y.saturating_sub(n); }
            ('B', []) => { let n = if p1 == 0 { 1 } else { p1 }; *self.cursor_y = (*self.cursor_y + n).min(self.rows.saturating_sub(1)); }
            ('C', []) => { let n = if p1 == 0 { 1 } else { p1 }; *self.cursor_x = (*self.cursor_x + n).min(self.cols.saturating_sub(1)); }
            ('D', []) => { let n = if p1 == 0 { 1 } else { p1 }; *self.cursor_x = self.cursor_x.saturating_sub(n); }
            ('H' | 'f', []) => {
                let row = if p1 == 0 { 1 } else { p1 };
                let col = ps.get(1).copied().unwrap_or(1).max(1);
                *self.cursor_y = (row - 1).min(self.rows.saturating_sub(1));
                *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1));
            }
            ('G', []) => { let col = if p1 == 0 { 1 } else { p1 }; *self.cursor_x = (col - 1).min(self.cols.saturating_sub(1)); }
            ('J', []) => self.erase_display(p1),
            ('K', []) => self.erase_line(p1),
            ('L', []) => {
                let n = if p1 == 0 { 1 } else { p1 as usize };
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
                let n = if p1 == 0 { 1 } else { p1 as usize };
                let y = *self.cursor_y as usize;
                let bot = *self.scroll_bottom as usize;
                for _ in 0..n {
                    if y <= bot && bot < self.cells.len() {
                        self.cells.remove(y);
                        self.cells.insert(bot, vec![TCell::default(); self.cols as usize]);
                    }
                }
            }
            ('S', []) => { let n = if p1 == 0 { 1 } else { p1 }; for _ in 0..n { self.scroll_up(); } }
            ('T', []) => { let n = if p1 == 0 { 1 } else { p1 }; for _ in 0..n { self.scroll_down(); } }
            ('m', []) => { if ps.is_empty() { self.set_sgr(&[0]); } else { self.set_sgr(&ps); } }
            ('r', []) => {
                let top = if p1 == 0 { 1 } else { p1 };
                let bot = ps.get(1).copied().unwrap_or(self.rows).min(self.rows);
                *self.scroll_top = top.saturating_sub(1);
                *self.scroll_bottom = bot.saturating_sub(1);
                *self.cursor_x = 0;
                *self.cursor_y = 0;
            }
            ('h', [b'?']) => { if p1 == 25 { *self.cursor_visible = true; } }
            ('l', [b'?']) => { if p1 == 25 { *self.cursor_visible = false; } }
            ('s', []) => { *self.saved_cursor = (*self.cursor_x, *self.cursor_y); }
            ('u', []) => { *self.cursor_x = self.saved_cursor.0; *self.cursor_y = self.saved_cursor.1; }
            ('P', []) => {
                let n = if p1 == 0 { 1 } else { p1 as usize };
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
                let n = if p1 == 0 { 1 } else { p1 as usize };
                let y = *self.cursor_y as usize;
                let x = *self.cursor_x as usize;
                if y < self.rows as usize {
                    let row = &mut self.cells[y];
                    for _ in 0..n { if x < row.len() { row.insert(x, TCell::default()); } }
                    row.truncate(self.cols as usize);
                }
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates) {
            (b'7', []) => { *self.saved_cursor = (*self.cursor_x, *self.cursor_y); }
            (b'8', []) => { *self.cursor_x = self.saved_cursor.0; *self.cursor_y = self.saved_cursor.1; }
            (b'D', []) => self.newline(),
            (b'M', []) => {
                if *self.cursor_y <= *self.scroll_top { self.scroll_down(); } else { *self.cursor_y -= 1; }
            }
            _ => {}
        }
    }
}
