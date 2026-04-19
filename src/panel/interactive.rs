use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use super::{Panel, PanelAction};
use crate::tab::TabManager;
use crate::termbuf::TermBuf;

#[derive(Default)]
pub struct InteractivePanel {
    pub tabs: TabManager,
    last_cols: u16,
    last_rows: u16,
}

impl InteractivePanel {
    /// Resize PTY to match panel dimensions.
    pub fn sync_size(&mut self, area: Rect) {
        let cols = area.width.saturating_sub(2);
        let rows = area.height.saturating_sub(3);
        if cols != self.last_cols || rows != self.last_rows {
            self.last_cols = cols;
            self.last_rows = rows;
            self.tabs.resize_active(cols, rows);
        }
    }

    /// Current inner dimensions for spawning new tabs.
    pub fn inner_size(&self) -> (u16, u16) {
        if self.last_cols > 0 && self.last_rows > 0 {
            (self.last_cols, self.last_rows)
        } else {
            (80, 24) // fallback only before first render
        }
    }
}

impl Panel for InteractivePanel {
    fn render(&self, frame: &mut Frame, area: Rect, focused: bool) {
        let chunks = Layout::vertical([Constraint::Length(1), Constraint::Min(1)]).split(area);

        render_tab_bar(frame, &self.tabs, chunks[0], focused);
        render_termbuf(frame, &self.tabs, chunks[1], focused);
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<PanelAction> {
        // Scroll-back: PgUp/PgDn (shells don't use these)
        match key.code {
            KeyCode::PageUp => {
                if let Some(tb) = self.tabs.active_termbuf_mut() {
                    tb.scroll_up(tb.rows() / 2);
                }
                return Ok(PanelAction::None);
            }
            KeyCode::PageDown => {
                if let Some(tb) = self.tabs.active_termbuf_mut() {
                    tb.scroll_down(tb.rows() / 2);
                }
                return Ok(PanelAction::None);
            }
            _ => {}
        }

        let bytes = key_to_bytes(key);
        if !bytes.is_empty() {
            // Snap to bottom on any input
            if let Some(tb) = self.tabs.active_termbuf_mut() {
                tb.scroll_to_bottom();
            }
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

fn render_termbuf(frame: &mut Frame, tabs: &TabManager, area: Rect, focused: bool) {
    let border_color = if focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let termbuf = match tabs.active_termbuf() {
        Some(tb) => tb,
        None => {
            let msg = Paragraph::new("No active terminal. Press Ctrl-S or Ctrl-K.");
            frame.render_widget(msg, inner);
            return;
        }
    };

    // Render termbuf cells directly into the ratatui buffer
    let buf = frame.buffer_mut();
    render_cells(buf, inner, termbuf);

    // Show cursor if focused and not scrolled back
    if focused && termbuf.scroll_offset == 0 && termbuf.cursor_visible {
        let (cr, cc) = termbuf.cursor();
        let cx = inner.x + cc as u16;
        let cy = inner.y + cr as u16;
        if cx < inner.right() && cy < inner.bottom() {
            frame.set_cursor_position((cx, cy));
        }
    }
}

fn render_cells(buf: &mut Buffer, area: Rect, termbuf: &TermBuf) {
    for row in 0..area.height as usize {
        if row >= termbuf.rows() {
            break;
        }
        let cells = termbuf.visible_row(row);
        for col in 0..area.width as usize {
            if col >= cells.len() {
                break;
            }
            let x = area.x + col as u16;
            let y = area.y + row as u16;
            if x < area.right() && y < area.bottom() {
                let cell = &cells[col];
                let buf_cell = &mut buf[(x, y)];
                buf_cell.set_char(cell.ch);
                buf_cell.set_style(cell.style);
            }
        }
    }
}

/// Convert a crossterm KeyEvent to raw bytes for the PTY.
fn key_to_bytes(key: KeyEvent) -> Vec<u8> {
    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
    let alt = key.modifiers.contains(KeyModifiers::ALT);

    if alt {
        // Alt+key sends ESC then the key
        let inner = key_code_to_bytes(key.code, ctrl);
        if inner.is_empty() {
            return Vec::new();
        }
        let mut out = vec![0x1b];
        out.extend_from_slice(&inner);
        return out;
    }

    key_code_to_bytes(key.code, ctrl)
}

fn key_code_to_bytes(code: KeyCode, ctrl: bool) -> Vec<u8> {
    match code {
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
        KeyCode::PageUp => b"\x1b[5~".to_vec(),
        KeyCode::PageDown => b"\x1b[6~".to_vec(),
        _ => Vec::new(),
    }
}
