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
use crate::tab::TabManager;

#[derive(Default)]
pub struct InteractivePanel {
    pub tabs: TabManager,
}

impl Panel for InteractivePanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);

        render_tab_bar(frame, &self.tabs, chunks[0], focused);
        render_output(frame, &self.tabs, chunks[1], focused);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        // Scroll-back keys (Shift+PgUp/PgDn/Home/End)
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

        // Forward raw keystrokes to the active backend
        let bytes = key_to_bytes(key);
        if !bytes.is_empty() {
            self.tabs.write_to_active(&bytes);
        }
        Ok(PanelAction::None)
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
    let scroll = tabs.active_scroll() as u16;

    let para = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));
    frame.render_widget(para, area);
}

/// Convert a crossterm KeyEvent to raw bytes for the PTY.
fn key_to_bytes(key: KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    match key.code {
        KeyCode::Char(c) if ctrl => {
            vec![(c as u8).wrapping_sub(b'a').wrapping_add(1)]
        }
        KeyCode::Char(c) => {
            let mut buf = [0u8; 4];
            c.encode_utf8(&mut buf).as_bytes().to_vec()
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => b"\x1b[A".to_vec(),
        KeyCode::Down => b"\x1b[B".to_vec(),
        KeyCode::Right => b"\x1b[C".to_vec(),
        KeyCode::Left => b"\x1b[D".to_vec(),
        KeyCode::Home => b"\x1b[H".to_vec(),
        KeyCode::End => b"\x1b[F".to_vec(),
        KeyCode::Delete => b"\x1b[3~".to_vec(),
        _ => Vec::new(),
    }
}
