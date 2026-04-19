use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Panel, PanelAction};
use crate::buffer::{BufferKind, OutputBuffer};

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

#[derive(Default)]
pub struct MainViewPanel {
    pub buffer: Option<OutputBuffer>,
    pub highlighted_lines: Vec<Line<'static>>,
    pub scroll: usize,
    pub mode: ViewMode,
    pub current_path: Option<String>,
    /// Line selection: (anchor, cursor). None = no selection.
    pub selection: Option<(usize, usize)>,
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

    fn extend_selection(&mut self, delta: isize) {
        let max = self.total_lines().saturating_sub(1);
        let (anchor, cursor) = self.selection.unwrap_or((self.scroll, self.scroll));
        let new_cursor = (cursor as isize + delta).clamp(0, max as isize) as usize;
        self.selection = Some((anchor, new_cursor));
        // Auto-scroll to keep cursor visible
        if new_cursor < self.scroll {
            self.scroll = new_cursor;
        } else if new_cursor >= self.scroll + 20 {
            self.scroll = new_cursor.saturating_sub(19);
        }
    }

    fn take_selection_text(&mut self) -> Option<String> {
        let (a, b) = self.selection.take()?;
        let start = a.min(b);
        let end = a.max(b);
        let content = self.buffer.as_ref()?.content.clone();
        let lines: Vec<&str> = content.lines().collect();
        let selected: Vec<&str> = lines
            .get(start..=end.min(lines.len().saturating_sub(1)))?
            .to_vec();
        Some(selected.join("\n"))
    }

    pub fn selection_range(&self) -> Option<(usize, usize)> {
        self.selection.map(|(a, b)| (a.min(b), a.max(b)))
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

        let title = match &self.buffer {
            Some(buf) => format!(" {} [{}] ", buf.title, self.mode.label()),
            None => " Main ".to_string(),
        };
        let line_info = format!(" L{}/{} ", self.scroll + 1, self.total_lines());
        let block = Block::default()
            .title(title)
            .title_bottom(Line::from(line_info).right_aligned())
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));

        if !self.highlighted_lines.is_empty() {
            let para = Paragraph::new(self.highlighted_lines.clone())
                .block(block)
                .scroll((self.scroll as u16, 0));
            frame.render_widget(para, area);
        } else {
            let text = match &self.buffer {
                Some(buf) => buf.content.as_str(),
                None => "No content. Open a file or pin tab output.",
            };
            let para = Paragraph::new(text)
                .block(block)
                .scroll((self.scroll as u16, 0));
            frame.render_widget(para, area);
        }

        // Highlight selected lines
        highlight_selection(frame, area, self.selection_range(), self.scroll);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        use crossterm::event::KeyModifiers;
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        if let Some(action) = self.handle_selection_key(shift, key.code) {
            return Ok(action);
        }
        self.handle_nav_key(key.code)
    }
}

impl MainViewPanel {
    fn handle_selection_key(&mut self, shift: bool, code: KeyCode) -> Option<PanelAction> {
        match (shift, code) {
            (true, KeyCode::Up) => {
                self.extend_selection(-1);
                Some(PanelAction::None)
            }
            (true, KeyCode::Down) => {
                self.extend_selection(1);
                Some(PanelAction::None)
            }
            (false, KeyCode::Enter) => self.take_selection_text().map(PanelAction::SendToKiro),
            (false, KeyCode::Esc) => {
                self.selection = None;
                Some(PanelAction::None)
            }
            _ => None,
        }
    }

    fn handle_nav_key(&mut self, code: KeyCode) -> Result<PanelAction> {
        match code {
            KeyCode::Up => self.scroll_by(-1, 20),
            KeyCode::Down => self.scroll_by(1, 20),
            KeyCode::PageUp => self.scroll_by(-20, 20),
            KeyCode::PageDown => self.scroll_by(20, 20),
            KeyCode::Home => self.scroll = 0,
            KeyCode::End => self.scroll = self.total_lines(),
            KeyCode::Tab => {
                if self.current_path.is_some() {
                    self.mode = self.mode.next();
                    return Ok(PanelAction::SwitchMode);
                }
            }
            _ => {}
        }
        Ok(PanelAction::None)
    }
}

fn highlight_selection(
    frame: &mut Frame,
    area: Rect,
    range: Option<(usize, usize)>,
    scroll: usize,
) {
    let (start, end) = match range {
        Some(r) => r,
        None => return,
    };
    let inner_y = area.y + 1;
    let inner_h = area.height.saturating_sub(2) as usize;
    let buf = frame.buffer_mut();
    for line in start..=end {
        if line < scroll || line >= scroll + inner_h {
            continue;
        }
        let y = inner_y + (line - scroll) as u16;
        for x in (area.x + 1)..area.right().saturating_sub(1) {
            if y < area.bottom() {
                buf[(x, y)].set_bg(Color::DarkGray);
            }
        }
    }
}
