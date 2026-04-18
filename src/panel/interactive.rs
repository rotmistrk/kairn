use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use super::{Panel, PanelAction};
use crate::input::{InputAction, InputLine, InputMode, SendTarget};
use crate::tab::TabManager;

pub struct InteractivePanel {
    pub tabs: TabManager,
    pub input: InputLine,
}

impl Default for InteractivePanel {
    fn default() -> Self {
        Self {
            tabs: TabManager::default(),
            input: InputLine::new(InputMode::default()),
        }
    }
}

impl Panel for InteractivePanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

        render_tab_bar(frame, &self.tabs, chunks[0], focused);
        render_output(frame, &self.tabs, chunks[1], focused);
        render_input(frame, &self.input, chunks[2], focused);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        // Scroll-back
        match (shift, key.code) {
            (true, KeyCode::PageUp) => {
                self.tabs.scroll_active(-20, 20);
                return Ok(PanelAction::None);
            }
            (true, KeyCode::PageDown) => {
                self.tabs.scroll_active(20, 20);
                return Ok(PanelAction::None);
            }
            (true, KeyCode::Home) => {
                self.tabs.scroll_active(-100_000, 20);
                return Ok(PanelAction::None);
            }
            (true, KeyCode::End) => {
                self.tabs.snap_to_bottom(20);
                return Ok(PanelAction::None);
            }
            _ => {}
        }

        // Input line handles the key
        match self.input.handle_key(key) {
            InputAction::None => {}
            InputAction::Send { text, target } => {
                self.dispatch_input(&text, target);
            }
        }
        Ok(PanelAction::None)
    }
}

impl InteractivePanel {
    fn dispatch_input(&mut self, text: &str, target: SendTarget) {
        match target {
            SendTarget::Kiro => {
                self.tabs.send_to_active_kiro(text);
            }
            SendTarget::Terminal => {
                if self.tabs.active_is_shell() {
                    self.tabs.run_in_active(text);
                } else {
                    self.tabs
                        .push_to_active(format!("[not a shell tab] {text}"));
                }
            }
        }
    }
}

fn render_tab_bar(frame: &mut Frame, tabs: &TabManager, area: Rect, focused: bool) {
    let labels = tabs.tab_labels();
    let spans: Vec<Span<'_>> = labels
        .iter()
        .flat_map(|(name, active)| {
            let style = if *active {
                Style::default()
                    .fg(if focused { Color::Cyan } else { Color::White })
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            let sep = Span::styled(" │ ", Style::default().fg(Color::DarkGray));
            vec![Span::styled(format!(" {name} "), style), sep]
        })
        .collect();
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn render_output(frame: &mut Frame, tabs: &TabManager, area: Rect, focused: bool) {
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let content = tabs.active_content();
    let lines: Vec<Line<'_>> = content
        .lines()
        .map(|l| {
            if l.starts_with('⚠') {
                Line::from(Span::styled(l, Style::default().fg(Color::Red)))
            } else if l.starts_with('$') {
                Line::from(Span::styled(l, Style::default().fg(Color::Yellow)))
            } else if l.starts_with('>') {
                Line::from(Span::styled(l, Style::default().fg(Color::LightCyan)))
            } else {
                Line::from(l)
            }
        })
        .collect();
    let scroll = tabs.active_scroll() as u16;

    let para = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(para, area);
}

fn render_input(frame: &mut Frame, input: &InputLine, area: Rect, focused: bool) {
    let mode = input.mode_indicator();
    let hint = if focused {
        "Enter→run  Alt-Enter→kiro"
    } else {
        ""
    };
    let title = Line::from(vec![
        Span::styled(
            format!(" [{mode}] "),
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
        .title(title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let para = Paragraph::new(input.text.as_str()).block(block);
    frame.render_widget(para, area);

    if focused && area.height >= 3 {
        let cx = (area.x + 1 + input.cursor as u16).min(area.right().saturating_sub(2));
        let cy = area.y + 1;
        if cy < area.bottom().saturating_sub(1) {
            frame.set_cursor_position((cx, cy));
        }
    }
}
