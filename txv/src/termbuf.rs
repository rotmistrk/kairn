// TermBuf — VTE-driven virtual terminal buffer.

use crate::cell::{Cell, Color, Style};
use crate::surface::Surface;
use unicode_width::UnicodeWidthChar;

/// Virtual terminal screen driven by ANSI escape sequences.
pub struct TermBuf {
    cols: u16,
    rows: u16,
    grid: Vec<Vec<Cell>>,
    scrollback: Vec<Vec<Cell>>,
    max_scrollback: usize,
    cursor_row: u16,
    cursor_col: u16,
    cursor_visible: bool,
    current_style: Style,
    scroll_offset: usize,
    responses: Vec<Vec<u8>>,
    saved_cursor: (u16, u16),
    saved_cursor_style: Style,
    scroll_top: u16,
    scroll_bottom: u16,
    alt_grid: Option<Vec<Vec<Cell>>>,
    alt_cursor: (u16, u16),
    bracketed_paste: bool,
}

impl TermBuf {
    /// Create a new terminal buffer.
    pub fn new(cols: u16, rows: u16) -> Self {
        let grid = (0..rows)
            .map(|_| vec![Cell::default(); cols as usize])
            .collect();
        Self {
            cols,
            rows,
            grid,
            scrollback: Vec::new(),
            max_scrollback: 10_000,
            cursor_row: 0,
            cursor_col: 0,
            cursor_visible: true,
            current_style: Style::default(),
            scroll_offset: 0,
            responses: Vec::new(),
            saved_cursor: (0, 0),
            saved_cursor_style: Style::default(),
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            alt_grid: None,
            alt_cursor: (0, 0),
            bracketed_paste: false,
        }
    }

    /// Feed raw bytes from PTY output. Parses via VTE and updates grid.
    pub fn process(&mut self, bytes: &[u8]) {
        self.scroll_offset = 0;
        let mut parser = PARSER.take();
        for &byte in bytes {
            parser.advance(self, byte);
        }
        PARSER.set(parser);
    }

    /// Resize the terminal. Preserves content where possible.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        let mut new_grid: Vec<Vec<Cell>> = (0..rows)
            .map(|_| vec![Cell::default(); cols as usize])
            .collect();
        let copy_rows = self.rows.min(rows) as usize;
        let copy_cols = self.cols.min(cols) as usize;
        for (r, new_row) in new_grid.iter_mut().enumerate().take(copy_rows) {
            new_row[..copy_cols].clone_from_slice(&self.grid[r][..copy_cols]);
        }
        self.grid = new_grid;
        self.cols = cols;
        self.rows = rows;
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
        self.scroll_top = 0;
        self.scroll_bottom = rows.saturating_sub(1);
    }

    /// Render current visible content to a surface.
    pub fn render_to(&self, surface: &mut Surface<'_>) {
        let sh = surface.height() as usize;
        for screen_row in 0..sh {
            let row = self.visible_row(screen_row);
            for (col, cell) in row.iter().enumerate() {
                if col as u16 >= surface.width() {
                    break;
                }
                surface.put(col as u16, screen_row as u16, cell.ch, cell.style);
            }
        }
    }

    /// Cursor position (col, row).
    pub fn cursor(&self) -> (u16, u16) {
        (self.cursor_col, self.cursor_row)
    }

    /// Whether cursor is visible.
    pub fn cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Drain response bytes (DA1, cursor position reports, etc.).
    pub fn drain_responses(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.responses)
    }

    /// Current scroll offset (0 = live screen).
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Set scroll offset for scrollback viewing.
    pub fn set_scroll_offset(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.scrollback.len());
    }

    /// Number of scrollback lines.
    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    /// Terminal columns.
    pub fn cols(&self) -> u16 {
        self.cols
    }

    /// Terminal rows.
    pub fn rows(&self) -> u16 {
        self.rows
    }
}

// Thread-local VTE parser to avoid allocation per process() call.
thread_local! {
    static PARSER: std::cell::Cell<vte::Parser> =
        std::cell::Cell::new(vte::Parser::new());
}

// --- Private helpers ---

impl TermBuf {
    fn visible_row(&self, screen_row: usize) -> &[Cell] {
        if self.scroll_offset == 0 {
            let r = screen_row.min(self.rows as usize - 1);
            return &self.grid[r];
        }
        let total = self.scrollback.len() + self.rows as usize;
        let start = total.saturating_sub(self.rows as usize + self.scroll_offset);
        let idx = start + screen_row;
        if idx < self.scrollback.len() {
            &self.scrollback[idx]
        } else {
            let gi = (idx - self.scrollback.len()).min(self.rows as usize - 1);
            &self.grid[gi]
        }
    }

    fn scroll_up_in_region(&mut self) {
        let top = self.scroll_top as usize;
        let bot = self.scroll_bottom as usize;
        if top >= self.grid.len() || bot >= self.grid.len() || top > bot {
            return;
        }
        let row = self.grid.remove(top);
        // Only add to scrollback if scrolling the full screen
        if self.scroll_top == 0 && self.scroll_bottom == self.rows - 1 {
            self.scrollback.push(row);
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }
        }
        self.grid
            .insert(bot, vec![Cell::default(); self.cols as usize]);
    }

    fn scroll_down_in_region(&mut self) {
        let top = self.scroll_top as usize;
        let bot = self.scroll_bottom as usize;
        if top >= self.grid.len() || bot >= self.grid.len() || top > bot {
            return;
        }
        self.grid.remove(bot);
        self.grid
            .insert(top, vec![Cell::default(); self.cols as usize]);
    }

    fn clear_row(&mut self, row: u16) {
        if (row as usize) < self.grid.len() {
            self.grid[row as usize] = vec![Cell::default(); self.cols as usize];
        }
    }

    fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                self.erase_line(0);
                for r in (self.cursor_row + 1)..self.rows {
                    self.clear_row(r);
                }
            }
            1 => {
                for r in 0..self.cursor_row {
                    self.clear_row(r);
                }
                self.erase_line(1);
            }
            2 | 3 => {
                for r in 0..self.rows {
                    self.clear_row(r);
                }
            }
            _ => {}
        }
    }

    fn erase_line(&mut self, mode: u16) {
        let r = self.cursor_row as usize;
        if r >= self.grid.len() {
            return;
        }
        let row = &mut self.grid[r];
        match mode {
            0 => {
                for c in row.iter_mut().skip(self.cursor_col as usize) {
                    *c = Cell::default();
                }
            }
            1 => {
                let end = (self.cursor_col as usize + 1).min(row.len());
                for c in row.iter_mut().take(end) {
                    *c = Cell::default();
                }
            }
            2 => {
                for c in row.iter_mut() {
                    *c = Cell::default();
                }
            }
            _ => {}
        }
    }

    fn enter_alt_screen(&mut self) {
        if self.alt_grid.is_some() {
            return;
        }
        self.alt_cursor = (self.cursor_col, self.cursor_row);
        let blank: Vec<Vec<Cell>> = (0..self.rows)
            .map(|_| vec![Cell::default(); self.cols as usize])
            .collect();
        self.alt_grid = Some(std::mem::replace(&mut self.grid, blank));
        self.cursor_row = 0;
        self.cursor_col = 0;
    }

    fn exit_alt_screen(&mut self) {
        if let Some(main) = self.alt_grid.take() {
            self.grid = main;
            self.cursor_col = self.alt_cursor.0;
            self.cursor_row = self.alt_cursor.1;
        }
    }
}

// --- VTE Perform implementation ---

impl vte::Perform for TermBuf {
    fn print(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row > self.scroll_bottom {
                self.scroll_up_in_region();
                self.cursor_row = self.scroll_bottom;
            }
        }
        let r = self.cursor_row as usize;
        let c_col = self.cursor_col as usize;
        if r < self.grid.len() && c_col < self.cols as usize {
            let w = c.width().unwrap_or(1) as u8;
            if w == 2 && c_col + 1 < self.cols as usize {
                self.grid[r][c_col] = Cell {
                    ch: c,
                    style: self.current_style,
                    width: 2,
                };
                self.grid[r][c_col + 1] = Cell {
                    ch: ' ',
                    style: self.current_style,
                    width: 0,
                };
                self.cursor_col += 2;
            } else if w == 2 {
                // Wide char at last column — use space
                self.grid[r][c_col] = Cell {
                    ch: ' ',
                    style: self.current_style,
                    width: 1,
                };
                self.cursor_col += 1;
            } else {
                self.grid[r][c_col] = Cell {
                    ch: c,
                    style: self.current_style,
                    width: w.max(1),
                };
                self.cursor_col += 1;
            }
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                if self.cursor_row >= self.scroll_bottom {
                    self.scroll_up_in_region();
                } else {
                    self.cursor_row += 1;
                }
            }
            b'\r' => self.cursor_col = 0,
            b'\t' => {
                let next = ((self.cursor_col / 8) + 1) * 8;
                self.cursor_col = next.min(self.cols.saturating_sub(1));
            }
            8 => {
                // Backspace
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
            7 => {} // Bell — ignore
            _ => {}
        }
    }

    fn csi_dispatch(
        &mut self,
        params: &vte::Params,
        intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        let p: Vec<u16> = params.iter().map(|s| s[0]).collect();
        if intermediates == [b'?'] {
            self.handle_dec_mode(&p, action);
        } else {
            self.handle_csi(&p, action);
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (intermediates, byte) {
            (_, b'7') => {
                self.saved_cursor = (self.cursor_col, self.cursor_row);
                self.saved_cursor_style = self.current_style;
            }
            (_, b'8') => {
                self.cursor_col = self.saved_cursor.0.min(self.cols.saturating_sub(1));
                self.cursor_row = self.saved_cursor.1.min(self.rows.saturating_sub(1));
                self.current_style = self.saved_cursor_style;
            }
            (_, b'M') => {
                // Reverse index — scroll down if at top of scroll region
                if self.cursor_row == self.scroll_top {
                    self.scroll_down_in_region();
                } else {
                    self.cursor_row = self.cursor_row.saturating_sub(1);
                }
            }
            _ => {}
        }
    }

    fn hook(&mut self, _: &vte::Params, _: &[u8], _: bool, _: char) {}
    fn put(&mut self, _: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _: &[&[u8]], _: bool) {}
}

// --- CSI and DEC mode handlers ---

impl TermBuf {
    fn handle_dec_mode(&mut self, p: &[u16], action: char) {
        let mode = p.first().copied().unwrap_or(0);
        match (mode, action) {
            (25, 'h') => self.cursor_visible = true,
            (25, 'l') => self.cursor_visible = false,
            (1049, 'h') => self.enter_alt_screen(),
            (1049, 'l') => self.exit_alt_screen(),
            (2004, 'h') => self.bracketed_paste = true,
            (2004, 'l') => self.bracketed_paste = false,
            _ => {}
        }
    }

    fn handle_csi(&mut self, p: &[u16], action: char) {
        match action {
            'm' => self.handle_sgr(p),
            'A' => self.cursor_up(p.first().copied().unwrap_or(1).max(1)),
            'B' => self.cursor_down(p.first().copied().unwrap_or(1).max(1)),
            'C' => self.cursor_forward(p.first().copied().unwrap_or(1).max(1)),
            'D' => self.cursor_back(p.first().copied().unwrap_or(1).max(1)),
            'E' => {
                // CNL — cursor next line
                let n = p.first().copied().unwrap_or(1).max(1);
                self.cursor_down(n);
                self.cursor_col = 0;
            }
            'F' => {
                // CPL — cursor previous line
                let n = p.first().copied().unwrap_or(1).max(1);
                self.cursor_up(n);
                self.cursor_col = 0;
            }
            'G' => {
                // CHA — cursor horizontal absolute
                let col = p.first().copied().unwrap_or(1);
                self.cursor_col = col.saturating_sub(1).min(self.cols.saturating_sub(1));
            }
            'H' | 'f' => {
                // CUP — cursor position
                let row = p.first().copied().unwrap_or(1);
                let col = p.get(1).copied().unwrap_or(1);
                self.cursor_row = row.saturating_sub(1).min(self.rows.saturating_sub(1));
                self.cursor_col = col.saturating_sub(1).min(self.cols.saturating_sub(1));
            }
            'J' => self.erase_display(p.first().copied().unwrap_or(0)),
            'K' => self.erase_line(p.first().copied().unwrap_or(0)),
            'L' => {
                // IL — insert lines
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                let r = self.cursor_row as usize;
                let bot = self.scroll_bottom as usize;
                for _ in 0..n {
                    if bot < self.grid.len() {
                        self.grid.remove(bot);
                    }
                    if r <= bot {
                        self.grid
                            .insert(r, vec![Cell::default(); self.cols as usize]);
                    }
                }
            }
            'M' => {
                // DL — delete lines
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                let r = self.cursor_row as usize;
                let bot = self.scroll_bottom as usize;
                for _ in 0..n {
                    if r < self.grid.len() {
                        self.grid.remove(r);
                    }
                    if bot < self.grid.len() + 1 {
                        self.grid
                            .insert(bot, vec![Cell::default(); self.cols as usize]);
                    }
                }
            }
            'S' => {
                // SU — scroll up
                let n = p.first().copied().unwrap_or(1).max(1);
                for _ in 0..n {
                    self.scroll_up_in_region();
                }
            }
            'T' => {
                // SD — scroll down
                let n = p.first().copied().unwrap_or(1).max(1);
                for _ in 0..n {
                    self.scroll_down_in_region();
                }
            }
            'd' => {
                // VPA — vertical position absolute
                let row = p.first().copied().unwrap_or(1);
                self.cursor_row = row.saturating_sub(1).min(self.rows.saturating_sub(1));
            }
            'n' if p.first().copied() == Some(6) => {
                // CPR — cursor position report
                let resp = format!("\x1b[{};{}R", self.cursor_row + 1, self.cursor_col + 1);
                self.responses.push(resp.into_bytes());
            }
            'r' => {
                // DECSTBM — set scroll region
                let top = p.first().copied().unwrap_or(1);
                let bot = p.get(1).copied().unwrap_or(self.rows);
                self.scroll_top = top.saturating_sub(1).min(self.rows.saturating_sub(1));
                self.scroll_bottom = bot.saturating_sub(1).min(self.rows.saturating_sub(1));
                if self.scroll_top > self.scroll_bottom {
                    self.scroll_top = 0;
                    self.scroll_bottom = self.rows.saturating_sub(1);
                }
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            's' => {
                self.saved_cursor = (self.cursor_col, self.cursor_row);
                self.saved_cursor_style = self.current_style;
            }
            'u' => {
                self.cursor_col = self.saved_cursor.0.min(self.cols.saturating_sub(1));
                self.cursor_row = self.saved_cursor.1.min(self.rows.saturating_sub(1));
                self.current_style = self.saved_cursor_style;
            }
            'P' => {
                // DCH — delete characters
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                let r = self.cursor_row as usize;
                let c = self.cursor_col as usize;
                if r < self.grid.len() {
                    let row = &mut self.grid[r];
                    for _ in 0..n {
                        if c < row.len() {
                            row.remove(c);
                            row.push(Cell::default());
                        }
                    }
                }
            }
            '@' => {
                // ICH — insert characters
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                let r = self.cursor_row as usize;
                let c = self.cursor_col as usize;
                if r < self.grid.len() {
                    let row = &mut self.grid[r];
                    for _ in 0..n {
                        if c < row.len() {
                            row.insert(c, Cell::default());
                            row.truncate(self.cols as usize);
                        }
                    }
                }
            }
            'X' => {
                // ECH — erase characters
                let n = p.first().copied().unwrap_or(1).max(1) as usize;
                let r = self.cursor_row as usize;
                let c = self.cursor_col as usize;
                if r < self.grid.len() {
                    let row = &mut self.grid[r];
                    let end = (c + n).min(row.len());
                    for cell in row[c..end].iter_mut() {
                        *cell = Cell::default();
                    }
                }
            }
            _ => {}
        }
    }

    fn cursor_up(&mut self, n: u16) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }

    fn cursor_down(&mut self, n: u16) {
        self.cursor_row = (self.cursor_row + n).min(self.rows.saturating_sub(1));
    }

    fn cursor_forward(&mut self, n: u16) {
        self.cursor_col = (self.cursor_col + n).min(self.cols.saturating_sub(1));
    }

    fn cursor_back(&mut self, n: u16) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }
}

// --- SGR (Select Graphic Rendition) ---

impl TermBuf {
    fn handle_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.current_style = Style::default();
            return;
        }
        let mut i = 0;
        while i < params.len() {
            i += self.apply_sgr(params, i);
        }
    }

    fn apply_sgr(&mut self, p: &[u16], i: usize) -> usize {
        match p[i] {
            0 => self.current_style = Style::default(),
            1 => self.current_style.attrs.bold = true,
            2 => self.current_style.attrs.dim = true,
            3 => self.current_style.attrs.italic = true,
            4 => self.current_style.attrs.underline = true,
            7 => self.current_style.attrs.reverse = true,
            9 => self.current_style.attrs.strikethrough = true,
            22 => {
                self.current_style.attrs.bold = false;
                self.current_style.attrs.dim = false;
            }
            23 => self.current_style.attrs.italic = false,
            24 => self.current_style.attrs.underline = false,
            27 => self.current_style.attrs.reverse = false,
            29 => self.current_style.attrs.strikethrough = false,
            30..=37 => self.current_style.fg = Color::Ansi(p[i] as u8 - 30),
            38 => return self.parse_extended_fg(p, i),
            39 => self.current_style.fg = Color::Reset,
            40..=47 => self.current_style.bg = Color::Ansi(p[i] as u8 - 40),
            48 => return self.parse_extended_bg(p, i),
            49 => self.current_style.bg = Color::Reset,
            90..=97 => self.current_style.fg = Color::Ansi(p[i] as u8 - 90 + 8),
            100..=107 => self.current_style.bg = Color::Ansi(p[i] as u8 - 100 + 8),
            _ => {}
        }
        1
    }

    fn parse_extended_fg(&mut self, p: &[u16], i: usize) -> usize {
        let start = i + 1;
        if start >= p.len() {
            return 1;
        }
        match p[start] {
            5 if start + 1 < p.len() => {
                self.current_style.fg = Color::Palette(p[start + 1] as u8);
                3
            }
            2 if start + 3 < p.len() => {
                self.current_style.fg =
                    Color::Rgb(p[start + 1] as u8, p[start + 2] as u8, p[start + 3] as u8);
                5
            }
            _ => 1,
        }
    }

    fn parse_extended_bg(&mut self, p: &[u16], i: usize) -> usize {
        let start = i + 1;
        if start >= p.len() {
            return 1;
        }
        match p[start] {
            5 if start + 1 < p.len() => {
                self.current_style.bg = Color::Palette(p[start + 1] as u8);
                3
            }
            2 if start + 3 < p.len() => {
                self.current_style.bg =
                    Color::Rgb(p[start + 1] as u8, p[start + 2] as u8, p[start + 3] as u8);
                5
            }
            _ => 1,
        }
    }
}

/// Extract all text (scrollback + grid) as a plain string.
pub fn extract_text(tb: &TermBuf) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(tb.scrollback.len() + tb.rows as usize);
    for row in &tb.scrollback {
        lines.push(row_to_string(row));
    }
    for row in &tb.grid {
        lines.push(row_to_string(row));
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

/// Extract text after the last prompt-like line.
pub fn extract_last_output(tb: &TermBuf) -> String {
    let mut lines: Vec<String> = Vec::with_capacity(tb.scrollback.len() + tb.rows as usize);
    for row in &tb.scrollback {
        lines.push(row_to_string(row));
    }
    for row in &tb.grid {
        lines.push(row_to_string(row));
    }
    while lines.last().is_some_and(|l| l.is_empty()) {
        lines.pop();
    }
    for i in (0..lines.len()).rev() {
        let s = lines[i].trim_start();
        if s.starts_with("> ") || s.starts_with("❯ ") || s.starts_with("$ ") || s.starts_with("% ")
        {
            return lines[i + 1..].join("\n");
        }
    }
    lines.join("\n")
}

fn row_to_string(row: &[Cell]) -> String {
    let s: String = row.iter().map(|c| c.ch).collect();
    s.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_text(tb: &TermBuf) -> String {
        let mut lines = Vec::new();
        for row in &tb.grid {
            lines.push(row_to_string(row));
        }
        while lines.last().is_some_and(|l| l.is_empty()) {
            lines.pop();
        }
        lines.join("\n")
    }

    #[test]
    fn basic_print() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"Hello");
        assert_eq!(grid_text(&tb), "Hello");
    }

    #[test]
    fn newline_and_cr() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"AB\r\nCD");
        assert_eq!(tb.grid[0][0].ch, 'A');
        assert_eq!(tb.grid[0][1].ch, 'B');
        assert_eq!(tb.grid[1][0].ch, 'C');
        assert_eq!(tb.grid[1][1].ch, 'D');
    }

    #[test]
    fn cursor_movement_cup() {
        let mut tb = TermBuf::new(10, 5);
        // CUP to row 3, col 5 (1-based)
        tb.process(b"\x1b[3;5HX");
        assert_eq!(tb.grid[2][4].ch, 'X');
    }

    #[test]
    fn cursor_up_down() {
        let mut tb = TermBuf::new(10, 5);
        tb.process(b"\x1b[3;1H"); // row 3
        tb.process(b"\x1b[2A"); // up 2 → row 1
        tb.process(b"U");
        assert_eq!(tb.grid[0][0].ch, 'U');
    }

    #[test]
    fn erase_display_below() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"AAAAAAAAAA");
        tb.process(b"\r\nBBBBBBBBBB");
        tb.process(b"\r\nCCCCCCCCCC");
        // Move to row 2, col 1 and erase below
        tb.process(b"\x1b[2;1H\x1b[J");
        assert_eq!(tb.grid[0][0].ch, 'A');
        assert_eq!(tb.grid[1][0].ch, ' ');
    }

    #[test]
    fn erase_line() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"ABCDEFGHIJ");
        tb.process(b"\x1b[1;5H"); // col 5
        tb.process(b"\x1b[K"); // erase to end
        assert_eq!(tb.grid[0][3].ch, 'D');
        assert_eq!(tb.grid[0][4].ch, ' ');
    }

    #[test]
    fn sgr_colors() {
        let mut tb = TermBuf::new(10, 3);
        // Set red foreground
        tb.process(b"\x1b[31mR");
        assert_eq!(tb.grid[0][0].style.fg, Color::Ansi(1));
        // Set 256-color bg
        tb.process(b"\x1b[48;5;100mB");
        assert_eq!(tb.grid[0][1].style.bg, Color::Palette(100));
        // Set RGB fg
        tb.process(b"\x1b[38;2;10;20;30mG");
        assert_eq!(tb.grid[0][2].style.fg, Color::Rgb(10, 20, 30));
    }

    #[test]
    fn sgr_attributes() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"\x1b[1;3;4mX");
        assert!(tb.grid[0][0].style.attrs.bold);
        assert!(tb.grid[0][0].style.attrs.italic);
        assert!(tb.grid[0][0].style.attrs.underline);
        // Reset
        tb.process(b"\x1b[0mY");
        assert!(!tb.grid[0][1].style.attrs.bold);
    }

    #[test]
    fn scroll_region() {
        let mut tb = TermBuf::new(10, 5);
        // Set scroll region rows 2-4 (1-based)
        tb.process(b"\x1b[2;4r");
        assert_eq!(tb.scroll_top, 1);
        assert_eq!(tb.scroll_bottom, 3);
    }

    #[test]
    fn cursor_position_report() {
        let mut tb = TermBuf::new(10, 5);
        tb.process(b"\x1b[3;7H"); // row 3, col 7
        tb.process(b"\x1b[6n"); // request CPR
        let responses = tb.drain_responses();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0], b"\x1b[3;7R");
    }

    #[test]
    fn cursor_visibility() {
        let mut tb = TermBuf::new(10, 5);
        assert!(tb.cursor_visible());
        tb.process(b"\x1b[?25l");
        assert!(!tb.cursor_visible());
        tb.process(b"\x1b[?25h");
        assert!(tb.cursor_visible());
    }

    #[test]
    fn scrollback() {
        let mut tb = TermBuf::new(10, 3);
        // Fill 3 rows then add more to trigger scrollback
        tb.process(b"AAA\r\nBBB\r\nCCC\r\nDDD");
        assert!(tb.scrollback_len() >= 1);
        assert_eq!(tb.grid[2][0].ch, 'D');
    }

    #[test]
    fn render_to_surface() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"Hello");
        let mut cells = vec![Cell::default(); 30];
        let mut surface = Surface::new(&mut cells, 10, 0, 0, 10, 3);
        tb.render_to(&mut surface);
        assert_eq!(cells[0].ch, 'H');
        assert_eq!(cells[4].ch, 'o');
    }

    #[test]
    fn resize_preserves_content() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"Hello");
        tb.resize(20, 5);
        assert_eq!(tb.cols(), 20);
        assert_eq!(tb.rows(), 5);
        assert_eq!(tb.grid[0][0].ch, 'H');
    }

    #[test]
    fn tab_stop() {
        let mut tb = TermBuf::new(20, 3);
        tb.process(b"A\tB");
        assert_eq!(tb.grid[0][0].ch, 'A');
        assert_eq!(tb.grid[0][8].ch, 'B');
    }

    #[test]
    fn backspace() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"AB\x08C");
        // Backspace moves cursor back, C overwrites B
        assert_eq!(tb.grid[0][0].ch, 'A');
        assert_eq!(tb.grid[0][1].ch, 'C');
    }

    #[test]
    fn wide_char_print() {
        let mut tb = TermBuf::new(10, 3);
        tb.process("漢".as_bytes());
        assert_eq!(tb.grid[0][0].ch, '漢');
        assert_eq!(tb.grid[0][0].width, 2);
        assert_eq!(tb.grid[0][1].width, 0);
    }

    #[test]
    fn extract_text_works() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"Hello\r\nWorld");
        let text = extract_text(&tb);
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }

    #[test]
    fn alt_screen() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"Main");
        tb.process(b"\x1b[?1049h"); // enter alt screen
        assert_eq!(tb.grid[0][0].ch, ' ');
        tb.process(b"Alt");
        assert_eq!(tb.grid[0][0].ch, 'A');
        tb.process(b"\x1b[?1049l"); // exit alt screen
        assert_eq!(tb.grid[0][0].ch, 'M');
    }

    #[test]
    fn cursor_save_restore() {
        let mut tb = TermBuf::new(10, 5);
        tb.process(b"\x1b[3;5H"); // row 3, col 5
        tb.process(b"\x1b[s"); // save
        tb.process(b"\x1b[1;1H"); // move to 1,1
        tb.process(b"\x1b[u"); // restore
        assert_eq!(tb.cursor(), (4, 2)); // 0-based: col 4, row 2
    }

    #[test]
    fn bright_colors() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"\x1b[91mR"); // bright red fg
        assert_eq!(tb.grid[0][0].style.fg, Color::Ansi(9));
        tb.process(b"\x1b[101mB"); // bright red bg
        assert_eq!(tb.grid[0][1].style.bg, Color::Ansi(9));
    }

    #[test]
    fn delete_chars() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"ABCDE");
        tb.process(b"\x1b[1;2H"); // col 2
        tb.process(b"\x1b[2P"); // delete 2 chars
        assert_eq!(tb.grid[0][0].ch, 'A');
        assert_eq!(tb.grid[0][1].ch, 'D');
        assert_eq!(tb.grid[0][2].ch, 'E');
    }

    #[test]
    fn insert_chars() {
        let mut tb = TermBuf::new(10, 3);
        tb.process(b"ABCDE");
        tb.process(b"\x1b[1;2H"); // col 2
        tb.process(b"\x1b[2@"); // insert 2 blanks
        assert_eq!(tb.grid[0][0].ch, 'A');
        assert_eq!(tb.grid[0][1].ch, ' ');
        assert_eq!(tb.grid[0][2].ch, ' ');
        assert_eq!(tb.grid[0][3].ch, 'B');
    }

    #[test]
    fn scroll_offset_viewing() {
        let mut tb = TermBuf::new(10, 3);
        // Push enough lines to create scrollback
        for i in 0..10 {
            let line = format!("Line{}\r\n", i);
            tb.process(line.as_bytes());
        }
        assert!(tb.scrollback_len() > 0);
        tb.set_scroll_offset(2);
        assert_eq!(tb.scroll_offset(), 2);
    }
}
