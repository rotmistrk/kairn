//! ResultsView — quickfix-style list for LSP refs, grep, build errors.
//!
//! Opens in the center panel. Enter navigates to the selected location.
//! n/p cycle through results. Reusable for any file:line:col results.

use std::path::PathBuf;

use txv_core::cell::{Attrs, Color, Style};
use txv_core::prelude::*;

use crate::commands::{CM_OPEN_FILE_FOCUS, OpenFileRequest};

/// A single result entry (file + location + context text).
#[derive(Debug, Clone)]
pub struct ResultEntry {
    pub path: PathBuf,
    pub line: u32,
    pub col: u32,
    pub text: String,
}

/// Quickfix-style results list view.
pub struct ResultsView {
    state: ViewState,
    entries: Vec<ResultEntry>,
    cursor: usize,
    scroll: usize,
    title: String,
}

impl ResultsView {
    pub fn new(title: &str, entries: Vec<ResultEntry>) -> Self {
        Self {
            state: ViewState::default(),
            entries,
            cursor: 0,
            scroll: 0,
            title: title.to_string(),
        }
    }

    /// Current entry (for next-error/prev-error navigation).
    pub fn current_entry(&self) -> Option<&ResultEntry> {
        self.entries.get(self.cursor)
    }

    /// Move to next entry, wrapping around.
    pub fn next(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = (self.cursor + 1) % self.entries.len();
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    /// Move to previous entry, wrapping around.
    pub fn prev(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = (self.cursor + self.entries.len() - 1) % self.entries.len();
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

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
    }

    fn open_current(&self, queue: &mut EventQueue) {
        if let Some(entry) = self.entries.get(self.cursor) {
            let req = OpenFileRequest::at(entry.path.clone(), entry.line, entry.col);
            queue.put_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
        }
    }
}

impl View for ResultsView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let dim = Style { fg: Color::Ansi(8), ..Style::default() };
        let normal = Style::default();
        let cursor_style = if self.state.is_focused() {
            Style { bg: Color::Ansi(4), attrs: Attrs { underline: true, ..Attrs::default() }, ..Style::default() }
        } else {
            Style { bg: Color::Ansi(8), ..Style::default() }
        };

        for row in 0..b.h as usize {
            let idx = self.scroll + row;
            let y = b.y + row as u16;
            if idx >= self.entries.len() {
                surface.hline(b.x, y, b.w, ' ', normal);
                continue;
            }
            let entry = &self.entries[idx];
            let style = if idx == self.cursor { cursor_style } else { normal };
            surface.hline(b.x, y, b.w, ' ', style);

            // Format: path:line: text
            let path_str = entry.path.to_string_lossy();
            let loc = format!("{}:{}:", path_str, entry.line + 1);
            surface.print(b.x, y, &loc, if idx == self.cursor { style } else { dim });
            let text_x = b.x + loc.len().min(b.w as usize) as u16;
            if text_x < b.x + b.w {
                surface.print(text_x, y, &entry.text, style);
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.next();
                HandleResult::Consumed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.prev();
                HandleResult::Consumed
            }
            KeyCode::Enter => {
                self.open_current(queue);
                HandleResult::Consumed
            }
            KeyCode::Char('n') => {
                self.next();
                self.open_current(queue);
                HandleResult::Consumed
            }
            KeyCode::Char('p') => {
                self.prev();
                self.open_current(queue);
                HandleResult::Consumed
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                queue.put_command(crate::commands::CM_TAB_CLOSE, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
