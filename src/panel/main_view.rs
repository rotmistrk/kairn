use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Panel, PanelAction};
use crate::buffer::{BufferKind, OutputBuffer};
use crate::input::{InputAction, InputLine, InputMode};

pub struct MainViewPanel {
    pub buffer: Option<OutputBuffer>,
    pub highlighted_lines: Vec<Line<'static>>,
    pub scroll: usize,
    pub input: InputLine,
}

impl MainViewPanel {
    pub fn new(mode: InputMode) -> Self {
        Self {
            buffer: None,
            highlighted_lines: Vec::new(),
            scroll: 0,
            input: InputLine::new(mode),
        }
    }

    pub fn set_buffer(&mut self, buf: OutputBuffer) {
        self.buffer = Some(buf);
        self.highlighted_lines.clear();
        self.scroll = 0;
    }

    /// Set pre-highlighted lines (called from App after highlighting).
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

    pub fn scroll_by(&mut self, delta: isize, viewport_h: usize) {
        let max = self.total_lines().saturating_sub(viewport_h);
        let new = (self.scroll as isize).saturating_add(delta);
        self.scroll = (new.max(0) as usize).min(max);
    }
}

impl Default for MainViewPanel {
    fn default() -> Self {
        Self::new(InputMode::default())
    }
}

impl Panel for MainViewPanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let border_color = if focused {
            Color::Cyan
        } else {
            Color::DarkGray
        };
        let chunks = Layout::vertical([Constraint::Min(1), Constraint::Length(3)]).split(area);

        render_content(self, frame, chunks[0], border_color);
        render_input_line(frame, chunks[1], &self.input, focused);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        // Scrolling keys handled before input line
        match key.code {
            KeyCode::Up => {
                self.scroll_by(-1, 20);
                return Ok(PanelAction::None);
            }
            KeyCode::Down => {
                self.scroll_by(1, 20);
                return Ok(PanelAction::None);
            }
            KeyCode::PageUp => {
                self.scroll_by(-20, 20);
                return Ok(PanelAction::None);
            }
            KeyCode::PageDown => {
                self.scroll_by(20, 20);
                return Ok(PanelAction::None);
            }
            KeyCode::Home => {
                self.scroll = 0;
                return Ok(PanelAction::None);
            }
            KeyCode::End => {
                self.scroll = self.total_lines();
                return Ok(PanelAction::None);
            }
            _ => {}
        }
        match self.input.handle_key(key) {
            InputAction::None => Ok(PanelAction::None),
            InputAction::Send { text, target } => Ok(PanelAction::SendInput { text, target }),
        }
    }
}

fn render_content(panel: &MainViewPanel, frame: &mut Frame, area: Rect, border_color: Color) {
    let title = match &panel.buffer {
        Some(buf) => format!(" {} ", buf.title),
        None => " Main ".to_string(),
    };
    let line_info = format!(" L{}/{} ", panel.scroll + 1, panel.total_lines());
    let block = Block::default()
        .title(title)
        .title_bottom(Line::from(line_info).right_aligned())
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    if !panel.highlighted_lines.is_empty() {
        let para = Paragraph::new(panel.highlighted_lines.clone())
            .block(block)
            .scroll((panel.scroll as u16, 0));
        frame.render_widget(para, area);
    } else {
        let text = match &panel.buffer {
            Some(buf) => buf.content.as_str(),
            None => "No content. Open a file or pin tab output.",
        };
        let para = Paragraph::new(text)
            .block(block)
            .scroll((panel.scroll as u16, 0));
        frame.render_widget(para, area);
    }
}

fn render_input_line(frame: &mut Frame, area: Rect, input: &InputLine, focused: bool) {
    let mode_str = input.mode_indicator();
    let hint = if focused {
        "Enter→kiro  Alt-Enter→shell"
    } else {
        ""
    };

    let title_line = Line::from(vec![
        Span::styled(
            format!(" [{mode_str}] "),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(hint, Style::default().fg(Color::DarkGray)),
    ]);

    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .title(title_line)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(input.text.as_str()).block(block);
    frame.render_widget(paragraph, area);

    if focused && area.height >= 3 {
        let cx = (area.x + 1 + input.cursor as u16).min(area.right().saturating_sub(2));
        let cy = area.y + 1;
        if cy < area.bottom().saturating_sub(1) {
            frame.set_cursor_position((cx, cy));
        }
    }
}
