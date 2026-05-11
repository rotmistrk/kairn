//! TermBuf — VTE-driven virtual terminal emulator that renders to a Surface.

mod vte_actions;
mod vte_handler;

use txv_core::cell::Style;
use txv_core::surface::Surface;

use vte_handler::Performer;

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
    responses: Vec<Vec<u8>>,
    /// When true, swallow all output until ESC \ (string terminator).
    swallow_until_st: bool,
    /// Saw ESC while in swallow mode (next byte might be \).
    swallow_saw_esc: bool,
}

#[derive(Clone)]
pub(super) struct TCell {
    pub ch: char,
    pub style: Style,
    #[allow(dead_code)]
    pub width: u8,
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
            responses: Vec::new(),
            swallow_until_st: false,
            swallow_saw_esc: false,
        }
    }

    /// Feed bytes into the terminal emulator.
    pub fn process(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            // Swallow content of ESC k ... ESC \ (tmux title sequence)
            if self.swallow_until_st {
                if self.swallow_saw_esc {
                    self.swallow_saw_esc = false;
                    if byte == b'\\' {
                        self.swallow_until_st = false;
                    }
                } else if byte == 0x1b {
                    self.swallow_saw_esc = true;
                }
                if byte == 0x07 {
                    self.swallow_until_st = false;
                }
                continue;
            }
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
                responses: &mut self.responses,
                swallow_flag: &mut self.swallow_until_st,
            };
            self.parser.advance(&mut performer, byte);
        }
    }

    /// Drain any pending response bytes (DA1, CPR replies).
    pub fn drain_responses(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.responses)
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

    /// Render terminal content to a Surface at a given offset.
    pub fn render_at(&self, surface: &mut Surface, ox: u16, oy: u16, w: u16, h: u16) {
        let rh = self.rows.min(h);
        let rw = self.cols.min(w);
        for y in 0..rh {
            for x in 0..rw {
                if ox + x < surface.width() && oy + y < surface.height() {
                    let tc = &self.cells[y as usize][x as usize];
                    surface.put(ox + x, oy + y, tc.ch, tc.style);
                }
            }
        }
    }

    pub fn cursor(&self) -> (u16, u16) {
        (self.cursor_x, self.cursor_y)
    }
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }
}

#[cfg(test)]
mod tests;
