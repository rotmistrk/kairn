use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Panel, PanelAction};
use crate::buffer::{BufferKind, OutputBuffer};

/// Info for rendering visual selection.
pub enum VisualInfo {
    Line((usize, usize)),
    Stream {
        anchor: (usize, usize),
        cursor: (usize, usize),
    },
    Block {
        rows: (usize, usize),
        cols: (usize, usize),
    },
}

fn safe_slice(s: &str, start: usize, end: usize) -> &str {
    let len = s.len();
    let s_byte = start.min(len);
    let e_byte = (end + 1).min(len);
    &s[s_byte..e_byte]
}

/// What the main panel is showing for the current file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    File,
    Diff,
    Log,
    Blame,
}

impl ViewMode {
    pub fn next(self) -> Self {
        match self {
            Self::File => Self::Diff,
            Self::Diff => Self::Log,
            Self::Log => Self::Blame,
            Self::Blame => Self::File,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Diff => "Diff",
            Self::Log => "Log",
            Self::Blame => "Blame",
        }
    }
}

/// Cursor/selection state for the main panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorMode {
    #[default]
    Off,
    /// Cursor visible, no selection.
    Normal,
    /// v: stream (character) selection.
    VisualStream,
    /// V: line selection.
    VisualLine,
    /// Ctrl-V: block (column) selection.
    VisualBlock,
}

#[derive(Default)]
pub struct MainViewPanel {
    pub buffer: Option<OutputBuffer>,
    pub highlighted_lines: Vec<Line<'static>>,
    pub scroll: usize,
    pub mode: ViewMode,
    pub current_path: Option<String>,
    /// Cursor mode.
    pub cursor_mode: CursorMode,
    /// Cursor position: (row, col).
    pub cursor: (usize, usize),
    /// Anchor position for visual modes: (row, col).
    pub anchor: (usize, usize),
    pub line_numbers: bool,
    /// Incremental search state.
    pub search_active: bool,
    pub search_query: String,
    /// (row, col) of each match.
    pub search_matches: Vec<(usize, usize)>,
    pub search_index: usize,
}

impl MainViewPanel {
    pub fn set_buffer(&mut self, buf: OutputBuffer) {
        if let BufferKind::FilePreview { ref path } = buf.kind {
            self.current_path = Some(path.clone());
        }
        self.buffer = Some(buf);
        self.highlighted_lines.clear();
        self.scroll = 0;
    }

    pub fn set_highlighted(&mut self, lines: Vec<Line<'static>>) {
        self.highlighted_lines = lines;
    }

    pub fn current_file_path(&self) -> Option<&str> {
        self.buffer.as_ref().and_then(|b| {
            if let BufferKind::FilePreview { ref path } = b.kind {
                Some(path.as_str())
            } else {
                None
            }
        })
    }

    fn total_lines(&self) -> usize {
        if !self.highlighted_lines.is_empty() {
            self.highlighted_lines.len()
        } else {
            self.buffer
                .as_ref()
                .map_or(1, |b| b.content.lines().count().max(1))
        }
    }

    fn move_cursor(&mut self, dr: isize, dc: isize) {
        let max_row = self.total_lines().saturating_sub(1);
        let r = (self.cursor.0 as isize + dr).clamp(0, max_row as isize) as usize;
        let c = (self.cursor.1 as isize + dc).max(0) as usize;
        self.cursor = (r, c);
        // Auto-scroll
        if r < self.scroll {
            self.scroll = r;
        } else if r >= self.scroll + 20 {
            self.scroll = r.saturating_sub(19);
        }
    }

    fn start_visual(&mut self, mode: CursorMode) {
        self.anchor = self.cursor;
        self.cursor_mode = mode;
    }

    fn take_selection_text(&mut self) -> Option<String> {
        let content = self.buffer.as_ref()?.content.clone();
        let lines: Vec<&str> = content.lines().collect();
        let text = match self.cursor_mode {
            CursorMode::VisualLine => {
                let (sr, er) = self.sel_rows();
                lines.get(sr..=er.min(lines.len() - 1))?.join("\n")
            }
            CursorMode::VisualStream => {
                let (sr, er) = self.sel_rows();
                if sr == er {
                    let (sc, ec) = self.sel_cols();
                    let line = lines.get(sr)?;
                    safe_slice(line, sc, ec).to_string()
                } else {
                    lines.get(sr..=er.min(lines.len() - 1))?.join("\n")
                }
            }
            CursorMode::VisualBlock => {
                let (sr, er) = self.sel_rows();
                let (sc, ec) = self.sel_cols();
                let mut out = Vec::new();
                for line in lines.iter().take(er.min(lines.len() - 1) + 1).skip(sr) {
                    out.push(safe_slice(line, sc, ec));
                }
                out.join("\n")
            }
            _ => return None,
        };
        self.cursor_mode = CursorMode::Normal;
        Some(text)
    }

    fn sel_rows(&self) -> (usize, usize) {
        let a = self.anchor.0;
        let b = self.cursor.0;
        (a.min(b), a.max(b))
    }

    fn sel_cols(&self) -> (usize, usize) {
        let a = self.anchor.1;
        let b = self.cursor.1;
        (a.min(b), a.max(b))
    }

    /// Get selection info for rendering.
    pub fn visual_info(&self) -> Option<VisualInfo> {
        match self.cursor_mode {
            CursorMode::VisualLine => Some(VisualInfo::Line(self.sel_rows())),
            CursorMode::VisualStream => Some(VisualInfo::Stream {
                anchor: self.anchor,
                cursor: self.cursor,
            }),
            CursorMode::VisualBlock => Some(VisualInfo::Block {
                rows: self.sel_rows(),
                cols: self.sel_cols(),
            }),
            _ => None,
        }
    }
    fn handle_search_input(&mut self, code: KeyCode) -> PanelAction {
        match code {
            KeyCode::Esc => {
                self.search_active = false;
            }
            KeyCode::Enter => {
                self.search_active = false;
                self.jump_to_match();
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.update_search_matches();
                self.jump_to_match();
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                self.update_search_matches();
                self.jump_to_match();
            }
            _ => {}
        }
        PanelAction::None
    }

    fn update_search_matches(&mut self) {
        self.search_matches.clear();
        if self.search_query.is_empty() {
            return;
        }
        let content = match &self.buffer {
            Some(b) => &b.content,
            None => return,
        };
        let query = &self.search_query;
        for (row, line) in content.lines().enumerate() {
            let lower_line = line.to_lowercase();
            let lower_query = query.to_lowercase();
            let mut start = 0;
            while let Some(pos) = lower_line[start..].find(&lower_query) {
                self.search_matches.push((row, start + pos));
                start += pos + lower_query.len();
            }
        }
        self.search_index = 0;
    }

    fn jump_to_match(&mut self) {
        if let Some(&(row, _)) = self.search_matches.get(self.search_index) {
            self.scroll = row.saturating_sub(5);
            if self.cursor_mode != CursorMode::Off {
                self.cursor = (row, 0);
            }
        }
    }

    fn next_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.search_index = (self.search_index + 1) % self.search_matches.len();
            self.jump_to_match();
        }
    }

    fn prev_match(&mut self) {
        if !self.search_matches.is_empty() {
            self.search_index = if self.search_index == 0 {
                self.search_matches.len() - 1
            } else {
                self.search_index - 1
            };
            self.jump_to_match();
        }
    }

    pub fn scroll_by(&mut self, delta: isize, viewport_h: usize) {
        let max = self.total_lines().saturating_sub(viewport_h);
        let new = (self.scroll as isize).saturating_add(delta);
        self.scroll = (new.max(0) as usize).min(max);
    }
}

impl Panel for MainViewPanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };

        let cursor_label = match self.cursor_mode {
            CursorMode::Off => "",
            CursorMode::Normal => " ●",
            CursorMode::VisualStream => " v",
            CursorMode::VisualLine => " V",
            CursorMode::VisualBlock => " ^V",
        };
        let title = match &self.buffer {
            Some(buf) => format!(" {} [{}]{cursor_label} ", buf.title, self.mode.label()),
            None => " Main ".to_string(),
        };
        let line_info = format!(" L{}/{} ", self.scroll + 1, self.total_lines());
        let block = Block::default()
            .title(title)
            .title_bottom(Line::from(line_info).right_aligned())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        // Clear inner area to prevent artifacts
        frame.render_widget(ratatui::widgets::Clear, inner);

        // Gutter width: 4 chars if line numbers on, 0 otherwise
        let gutter_w = if self.line_numbers && self.buffer.is_some() {
            4u16
        } else {
            0
        };
        let content_area = Rect::new(
            inner.x + gutter_w,
            inner.y,
            inner.width.saturating_sub(gutter_w),
            inner.height,
        );

        if !self.highlighted_lines.is_empty() {
            let para =
                Paragraph::new(self.highlighted_lines.clone()).scroll((self.scroll as u16, 0));
            frame.render_widget(para, content_area);
        } else {
            let text = match &self.buffer {
                Some(buf) => buf.content.as_str(),
                None => "No content. Open a file or pin tab output.",
            };
            let para = Paragraph::new(text).scroll((self.scroll as u16, 0));
            frame.render_widget(para, content_area);
        }

        // Highlight cursor line and visual selection
        render_cursor_and_selection(frame, area, self);

        // Highlight search matches
        render_search_matches(frame, area, self);

        // Search input bar
        if self.search_active {
            render_search_bar(frame, area, self);
        }

        // Line numbers gutter
        if self.line_numbers && self.buffer.is_some() {
            render_line_numbers(frame, area, self.scroll, self.total_lines());
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        use crossterm::event::KeyModifiers;
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);

        // Search input mode
        if self.search_active {
            return Ok(self.handle_search_input(key.code));
        }

        // n/N for next/prev match (when not in search input)
        if self.cursor_mode != CursorMode::Off || !self.search_matches.is_empty() {
            match key.code {
                KeyCode::Char('n') if !ctrl => {
                    self.next_match();
                    return Ok(PanelAction::None);
                }
                KeyCode::Char('N') => {
                    self.prev_match();
                    return Ok(PanelAction::None);
                }
                KeyCode::Char('/') if !ctrl => {
                    self.search_active = true;
                    self.search_query.clear();
                    return Ok(PanelAction::None);
                }
                _ => {}
            }
        }

        // / to start search (also works in scroll mode)
        if key.code == KeyCode::Char('/') && !ctrl {
            self.search_active = true;
            self.search_query.clear();
            self.cursor_mode = CursorMode::Normal;
            self.cursor = (self.scroll, 0);
            return Ok(PanelAction::None);
        }

        // Space toggles cursor mode
        if key.code == KeyCode::Char(' ') && !ctrl && !alt {
            self.cursor_mode = match self.cursor_mode {
                CursorMode::Off => {
                    self.cursor = (self.scroll, 0);
                    CursorMode::Normal
                }
                _ => CursorMode::Off,
            };
            return Ok(PanelAction::None);
        }

        // In cursor mode: handle cursor keys and visual modes
        if self.cursor_mode != CursorMode::Off {
            return self.handle_cursor_key(key.code, ctrl);
        }

        // Scroll mode (default)
        self.handle_nav_key(key.code)
    }
}

impl MainViewPanel {
    fn handle_cursor_key(&mut self, code: KeyCode, ctrl: bool) -> Result<PanelAction> {
        match code {
            KeyCode::Up => self.move_cursor(-1, 0),
            KeyCode::Down => self.move_cursor(1, 0),
            KeyCode::Left => self.move_cursor(0, -1),
            KeyCode::Right => self.move_cursor(0, 1),
            KeyCode::Home => self.cursor.1 = 0,
            KeyCode::End => self.cursor.1 = 999,
            KeyCode::PageUp => self.move_cursor(-20, 0),
            KeyCode::PageDown => self.move_cursor(20, 0),
            KeyCode::Char('v') if !ctrl => self.start_visual(CursorMode::VisualStream),
            KeyCode::Char('V') => self.start_visual(CursorMode::VisualLine),
            KeyCode::Char('v') if ctrl => self.start_visual(CursorMode::VisualBlock),
            KeyCode::Enter => {
                if let Some(text) = self.take_selection_text() {
                    return Ok(PanelAction::SendToKiro(text));
                }
            }
            KeyCode::Esc => self.cursor_mode = CursorMode::Normal,
            _ => {}
        }
        Ok(PanelAction::None)
    }

    fn handle_nav_key(&mut self, code: KeyCode) -> Result<PanelAction> {
        match code {
            KeyCode::Up => self.scroll_by(-1, 20),
            KeyCode::Down => self.scroll_by(1, 20),
            KeyCode::PageUp => self.scroll_by(-20, 20),
            KeyCode::PageDown => self.scroll_by(20, 20),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = self.total_lines(),
            _ => {}
        }
        Ok(PanelAction::None)
    }
}

fn render_cursor_and_selection(frame: &mut Frame, area: Rect, panel: &MainViewPanel) {
    let inner_y = area.y + 1;
    let inner_h = area.height.saturating_sub(2) as usize;
    let inner_x = area.x + 1;
    let inner_w = area.width.saturating_sub(2) as usize;
    let scroll = panel.scroll;
    let buf = frame.buffer_mut();

    // Cursor line highlight
    if panel.cursor_mode != CursorMode::Off {
        let cr = panel.cursor.0;
        if cr >= scroll && cr < scroll + inner_h {
            let y = inner_y + (cr - scroll) as u16;
            for x in inner_x..inner_x + inner_w as u16 {
                if y < area.bottom() && x < area.right() {
                    buf[(x, y)].set_bg(Color::Indexed(236));
                }
            }
        }
    }

    // Visual selection highlight
    if let Some(vi) = panel.visual_info() {
        match vi {
            VisualInfo::Line((sr, er)) => {
                highlight_rows(buf, area, inner_y, inner_w, scroll, inner_h, sr, er);
            }
            VisualInfo::Stream { .. } => {
                let (sr, er) = panel.sel_rows();
                highlight_rows(buf, area, inner_y, inner_w, scroll, inner_h, sr, er);
            }
            VisualInfo::Block { rows, cols } => {
                for r in rows.0..=rows.1 {
                    if r < scroll || r >= scroll + inner_h {
                        continue;
                    }
                    let y = inner_y + (r - scroll) as u16;
                    for c in cols.0..=cols.1 {
                        let x = inner_x + c as u16;
                        if y < area.bottom() && x < area.right() {
                            buf[(x, y)].set_bg(Color::DarkGray);
                        }
                    }
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn highlight_rows(
    buf: &mut ratatui::buffer::Buffer,
    area: Rect,
    inner_y: u16,
    inner_w: usize,
    scroll: usize,
    inner_h: usize,
    start: usize,
    end: usize,
) {
    for line in start..=end {
        if line < scroll || line >= scroll + inner_h {
            continue;
        }
        let y = inner_y + (line - scroll) as u16;
        let x_start = area.x + 1;
        for x in x_start..x_start + inner_w as u16 {
            if y < area.bottom() && x < area.right() {
                buf[(x, y)].set_bg(Color::DarkGray);
            }
        }
    }
}

fn render_line_numbers(frame: &mut Frame, area: Rect, scroll: usize, total: usize) {
    let inner_y = area.y + 1;
    let inner_h = area.height.saturating_sub(2) as usize;
    let gutter_w = 4u16; // "123 " = 4 chars
    let gutter_style = Style::default().fg(Color::DarkGray);
    let buf = frame.buffer_mut();

    for row in 0..inner_h {
        let line_no = scroll + row + 1;
        if line_no > total {
            break;
        }
        let y = inner_y + row as u16;
        let num_str = format!("{line_no:>3} ");
        for (i, ch) in num_str.chars().enumerate() {
            let x = area.x + 1 + i as u16;
            if x < area.x + 1 + gutter_w && y < area.bottom() {
                buf[(x, y)].set_char(ch);
                buf[(x, y)].set_style(gutter_style);
            }
        }
    }
}

fn render_search_matches(frame: &mut Frame, area: Rect, panel: &MainViewPanel) {
    if panel.search_query.is_empty() {
        return;
    }
    let gutter_w = if panel.line_numbers && panel.buffer.is_some() {
        4u16
    } else {
        0
    };
    let inner_y = area.y + 1;
    let inner_x = area.x + 1 + gutter_w;
    let inner_h = area.height.saturating_sub(2) as usize;
    let qlen = panel.search_query.len();
    let buf = frame.buffer_mut();

    for (i, &(row, col)) in panel.search_matches.iter().enumerate() {
        if row < panel.scroll || row >= panel.scroll + inner_h {
            continue;
        }
        let y = inner_y + (row - panel.scroll) as u16;
        let bg = if i == panel.search_index {
            Color::Yellow
        } else {
            Color::Indexed(58) // dark yellow
        };
        for c in 0..qlen {
            let x = inner_x + (col + c) as u16;
            if x < area.right().saturating_sub(1) && y < area.bottom() {
                buf[(x, y)].set_bg(bg);
                if i == panel.search_index {
                    buf[(x, y)].set_fg(Color::Black);
                }
            }
        }
    }
}

fn render_search_bar(frame: &mut Frame, area: Rect, panel: &MainViewPanel) {
    let y = area.bottom().saturating_sub(2);
    let w = area.width.saturating_sub(2);
    let bar_area = Rect::new(area.x + 1, y, w, 1);

    let match_info = if panel.search_matches.is_empty() {
        "no matches".to_string()
    } else {
        format!("{}/{}", panel.search_index + 1, panel.search_matches.len())
    };

    let line = Line::from(vec![
        Span::styled("/", Style::default().fg(Color::Yellow)),
        Span::styled(
            panel.search_query.as_str(),
            Style::default().fg(Color::White).bg(Color::DarkGray),
        ),
        Span::styled(
            format!("  [{match_info}]"),
            Style::default().fg(Color::DarkGray),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), bar_area);
}
