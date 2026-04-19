// Virtual terminal buffer: grid of styled cells driven by vte parser.

use ratatui::style::{Color, Modifier, Style};

/// A single cell in the terminal grid.
#[derive(Debug, Clone)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: ' ',
            style: Style::default(),
        }
    }
}

/// Virtual terminal screen buffer with scrollback.
pub struct TermBuf {
    cols: usize,
    rows: usize,
    /// Active screen grid (rows x cols).
    grid: Vec<Vec<Cell>>,
    /// Scrollback history (oldest first).
    scrollback: Vec<Vec<Cell>>,
    /// Max scrollback lines.
    max_scrollback: usize,
    /// Cursor position.
    cursor_row: usize,
    cursor_col: usize,
    /// Current style for new characters.
    current_style: Style,
    /// Scroll offset (0 = showing live screen).
    pub scroll_offset: usize,
    /// Whether cursor should be visible.
    pub cursor_visible: bool,
}

impl TermBuf {
    pub fn new(cols: usize, rows: usize) -> Self {
        let grid = vec![vec![Cell::default(); cols]; rows];
        Self {
            cols,
            rows,
            grid,
            scrollback: Vec::new(),
            max_scrollback: 10_000,
            cursor_row: 0,
            cursor_col: 0,
            current_style: Style::default(),
            scroll_offset: 0,
            cursor_visible: true,
        }
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn cursor(&self) -> (usize, usize) {
        (self.cursor_row, self.cursor_col)
    }

    /// Resize the buffer. Preserves content where possible.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.cols = cols;
        self.rows = rows;
        self.grid.resize_with(rows, || vec![Cell::default(); cols]);
        for row in &mut self.grid {
            row.resize(cols, Cell::default());
        }
        self.cursor_row = self.cursor_row.min(rows.saturating_sub(1));
        self.cursor_col = self.cursor_col.min(cols.saturating_sub(1));
    }

    /// Get a row for rendering. Negative offsets read from scrollback.
    pub fn visible_row(&self, screen_row: usize) -> &[Cell] {
        if self.scroll_offset == 0 {
            return &self.grid[screen_row.min(self.rows - 1)];
        }
        let total = self.scrollback.len() + self.rows;
        let start = total.saturating_sub(self.rows + self.scroll_offset);
        let idx = start + screen_row;
        if idx < self.scrollback.len() {
            &self.scrollback[idx]
        } else {
            let grid_idx = idx - self.scrollback.len();
            &self.grid[grid_idx.min(self.rows - 1)]
        }
    }

    pub fn total_lines(&self) -> usize {
        self.scrollback.len() + self.rows
    }

    pub fn scroll_up(&mut self, n: usize) {
        let max = self.scrollback.len();
        self.scroll_offset = (self.scroll_offset + n).min(max);
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(n);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    /// Feed raw bytes from PTY through the vte parser.
    pub fn process(&mut self, data: &[u8]) {
        // Auto-snap to bottom on new output
        self.scroll_offset = 0;
        let mut parser = PARSER.take();
        for byte in data {
            parser.advance(self, *byte);
        }
        PARSER.set(parser);
    }

    fn scroll_screen_up(&mut self) {
        if !self.grid.is_empty() {
            let row = self.grid.remove(0);
            self.scrollback.push(row);
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
            }
            self.grid.push(vec![Cell::default(); self.cols]);
        }
    }

    fn clear_row(&mut self, row: usize) {
        if row < self.rows {
            self.grid[row] = vec![Cell::default(); self.cols];
        }
    }
}

// Thread-local vte parser (avoids allocation per process() call)
thread_local! {
    static PARSER: std::cell::Cell<vte::Parser> =
        std::cell::Cell::new(vte::Parser::new());
}

impl vte::Perform for TermBuf {
    fn print(&mut self, c: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
            if self.cursor_row >= self.rows {
                self.scroll_screen_up();
                self.cursor_row = self.rows - 1;
            }
        }
        if self.cursor_row < self.rows && self.cursor_col < self.cols {
            self.grid[self.cursor_row][self.cursor_col] = Cell {
                ch: c,
                style: self.current_style,
            };
            self.cursor_col += 1;
        }
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.cursor_row += 1;
                if self.cursor_row >= self.rows {
                    self.scroll_screen_up();
                    self.cursor_row = self.rows - 1;
                }
            }
            b'\r' => self.cursor_col = 0,
            b'\t' => {
                let next_tab = (self.cursor_col / 8 + 1) * 8;
                self.cursor_col = next_tab.min(self.cols - 1);
            }
            8 => {
                // Backspace
                self.cursor_col = self.cursor_col.saturating_sub(1);
            }
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

        // DEC private modes: ESC [ ? N h/l
        if intermediates == [b'?'] {
            let mode = p.first().copied().unwrap_or(0);
            match (mode, action) {
                (25, 'h') => self.cursor_visible = true,
                (25, 'l') => self.cursor_visible = false,
                _ => {} // ignore other private modes
            }
            return;
        }

        match action {
            'm' => self.handle_sgr(&p),
            'A' => self.cursor_up(p.first().copied().unwrap_or(1) as usize),
            'B' => self.cursor_down(p.first().copied().unwrap_or(1) as usize),
            'C' => self.cursor_forward(p.first().copied().unwrap_or(1) as usize),
            'D' => self.cursor_back(p.first().copied().unwrap_or(1) as usize),
            'H' | 'f' => {
                let row = p.first().copied().unwrap_or(1) as usize;
                let col = p.get(1).copied().unwrap_or(1) as usize;
                self.cursor_row = row.saturating_sub(1).min(self.rows - 1);
                self.cursor_col = col.saturating_sub(1).min(self.cols - 1);
            }
            'J' => {
                let mode = p.first().copied().unwrap_or(0);
                self.erase_display(mode);
            }
            'K' => {
                let mode = p.first().copied().unwrap_or(0);
                self.erase_line(mode);
            }
            'G' => {
                let col = p.first().copied().unwrap_or(1) as usize;
                self.cursor_col = col.saturating_sub(1).min(self.cols - 1);
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
    }
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}
    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

impl TermBuf {
    fn cursor_up(&mut self, n: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(n);
    }

    fn cursor_down(&mut self, n: usize) {
        self.cursor_row = (self.cursor_row + n).min(self.rows - 1);
    }

    fn cursor_forward(&mut self, n: usize) {
        self.cursor_col = (self.cursor_col + n).min(self.cols - 1);
    }

    fn cursor_back(&mut self, n: usize) {
        self.cursor_col = self.cursor_col.saturating_sub(n);
    }

    fn erase_display(&mut self, mode: u16) {
        match mode {
            0 => {
                // Clear from cursor to end
                self.erase_line(0);
                for r in (self.cursor_row + 1)..self.rows {
                    self.clear_row(r);
                }
            }
            1 => {
                // Clear from start to cursor
                for r in 0..self.cursor_row {
                    self.clear_row(r);
                }
            }
            2 | 3 => {
                // Clear entire screen
                for r in 0..self.rows {
                    self.clear_row(r);
                }
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            _ => {}
        }
    }

    fn erase_line(&mut self, mode: u16) {
        if self.cursor_row >= self.rows {
            return;
        }
        let row = &mut self.grid[self.cursor_row];
        match mode {
            0 => {
                for c in row.iter_mut().skip(self.cursor_col) {
                    *c = Cell::default();
                }
            }
            1 => {
                for c in row.iter_mut().take(self.cursor_col + 1) {
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

    fn handle_sgr(&mut self, params: &[u16]) {
        if params.is_empty() {
            self.current_style = Style::default();
            return;
        }
        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => self.current_style = Style::default(),
                1 => self.current_style = self.current_style.add_modifier(Modifier::BOLD),
                3 => self.current_style = self.current_style.add_modifier(Modifier::ITALIC),
                4 => self.current_style = self.current_style.add_modifier(Modifier::UNDERLINED),
                7 => self.current_style = self.current_style.add_modifier(Modifier::REVERSED),
                22 => self.current_style = self.current_style.remove_modifier(Modifier::BOLD),
                23 => self.current_style = self.current_style.remove_modifier(Modifier::ITALIC),
                24 => self.current_style = self.current_style.remove_modifier(Modifier::UNDERLINED),
                27 => self.current_style = self.current_style.remove_modifier(Modifier::REVERSED),
                30..=37 => self.current_style = self.current_style.fg(ansi_color(params[i] - 30)),
                38 => {
                    i += 1;
                    if let Some(c) = parse_extended_color(params, &mut i) {
                        self.current_style = self.current_style.fg(c);
                    }
                    continue;
                }
                39 => self.current_style = self.current_style.fg(Color::Reset),
                40..=47 => self.current_style = self.current_style.bg(ansi_color(params[i] - 40)),
                48 => {
                    i += 1;
                    if let Some(c) = parse_extended_color(params, &mut i) {
                        self.current_style = self.current_style.bg(c);
                    }
                    continue;
                }
                49 => self.current_style = self.current_style.bg(Color::Reset),
                90..=97 => {
                    self.current_style = self.current_style.fg(ansi_bright_color(params[i] - 90))
                }
                100..=107 => {
                    self.current_style = self.current_style.bg(ansi_bright_color(params[i] - 100))
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn ansi_color(n: u16) -> Color {
    match n {
        0 => Color::Black,
        1 => Color::Red,
        2 => Color::Green,
        3 => Color::Yellow,
        4 => Color::Blue,
        5 => Color::Magenta,
        6 => Color::Cyan,
        7 => Color::White,
        _ => Color::White,
    }
}

fn ansi_bright_color(n: u16) -> Color {
    match n {
        0 => Color::DarkGray,
        1 => Color::LightRed,
        2 => Color::LightGreen,
        3 => Color::LightYellow,
        4 => Color::LightBlue,
        5 => Color::LightMagenta,
        6 => Color::LightCyan,
        7 => Color::White,
        _ => Color::White,
    }
}

fn parse_extended_color(params: &[u16], i: &mut usize) -> Option<Color> {
    if *i >= params.len() {
        return None;
    }
    match params[*i] {
        5 => {
            // 256-color: 38;5;N
            *i += 1;
            if *i < params.len() {
                let n = params[*i];
                *i += 1;
                Some(Color::Indexed(n as u8))
            } else {
                None
            }
        }
        2 => {
            // RGB: 38;2;R;G;B
            *i += 1;
            if *i + 2 < params.len() {
                let r = params[*i] as u8;
                let g = params[*i + 1] as u8;
                let b = params[*i + 2] as u8;
                *i += 3;
                Some(Color::Rgb(r, g, b))
            } else {
                None
            }
        }
        _ => None,
    }
}
