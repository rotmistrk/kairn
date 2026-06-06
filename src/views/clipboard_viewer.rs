//! ClipboardViewer — tool panel showing clipboard ring entries.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::clipboard_ring::ClipboardHandle;

pub struct ClipboardViewer {
    state: ViewState,
    clipboard: ClipboardHandle,
    scroll: usize,
    cursor: usize,
}

impl ClipboardViewer {
    pub fn new(clipboard: ClipboardHandle) -> Self {
        Self {
            state: ViewState::default(),
            clipboard,
            scroll: 0,
            cursor: 0,
        }
    }
}

impl View for ClipboardViewer {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        "Clipboard"
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let Ok(ring) = self.clipboard.lock() else {
            return;
        };
        let entries = ring.entries();
        let normal = Style::default();
        let cur = if self.state.is_focused() {
            palette().style(StyleId::CursorFocused)
        } else {
            palette().style(StyleId::CursorUnfocused)
        };
        for row in 0..h as usize {
            let idx = self.scroll + row;
            let y = row as u16;
            let style = if idx == self.cursor {
                cur
            } else {
                normal
            };
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            if let Some(entry) = entries.get(idx) {
                let line = entry.text.lines().next().unwrap_or("");
                let disp = if entry.line_count > 1 {
                    format!("[{}L] {}", entry.line_count, line)
                } else {
                    line.to_string()
                };
                let trunc: String = disp.chars().take(w as usize).collect();
                self.state.buffer_mut().print(0, y, &trunc, style);
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        let entry_count = self.clipboard.lock().map(|r| r.len()).unwrap_or(0);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.sync_scroll();
                }
                HandleResult::Consumed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.cursor + 1 < entry_count {
                    self.cursor += 1;
                    self.sync_scroll();
                }
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                if let Ok(mut ring) = self.clipboard.lock() {
                    ring.select(self.cursor);
                }
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

impl ClipboardViewer {
    fn sync_scroll(&mut self) {
        let h = self.state.bounds().h as usize;
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
        self.state.mark_dirty();
    }
}
