//! ResultsView — quickfix-style list for LSP refs, grep, build errors.
//!
//! Opens in the center panel. Enter navigates to the selected location.
//! n/p cycle through results. Supports async streaming of results.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use txv_core::cell::{Attrs, Color, Style};
use txv_core::prelude::*;

use crate::commands::{CM_OPEN_FILE, OpenFileRequest};

/// A single result entry (file + location + context text).
#[derive(Debug, Clone)]
pub struct ResultEntry {
    pub path: PathBuf,
    pub line: u32,
    pub col: u32,
    pub text: String,
}

/// Search state for progress display.
enum SearchState {
    Searching(mpsc::Receiver<Vec<ResultEntry>>),
    Done,
    NoMatches,
}

/// Quickfix-style results list view.
pub struct ResultsView {
    state: ViewState,
    entries: Vec<ResultEntry>,
    cursor: usize,
    scroll: usize,
    title: String,
    search: SearchState,
    root: PathBuf,
}

impl ResultsView {
    /// Create with pre-populated entries (LSP refs, build errors).
    pub fn new(title: &str, entries: Vec<ResultEntry>) -> Self {
        let search = if entries.is_empty() {
            SearchState::NoMatches
        } else {
            SearchState::Done
        };
        Self {
            state: ViewState::default(),
            entries,
            cursor: 0,
            scroll: 0,
            title: title.to_string(),
            search,
            root: PathBuf::new(),
        }
    }

    /// Create with a streaming receiver (grep).
    pub fn streaming(title: &str, rx: mpsc::Receiver<Vec<ResultEntry>>, root: &Path) -> Self {
        Self {
            state: ViewState::default(),
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            title: title.to_string(),
            search: SearchState::Searching(rx),
            root: root.to_path_buf(),
        }
    }

    pub fn current_entry(&self) -> Option<&ResultEntry> {
        self.entries.get(self.cursor)
    }

    pub fn next(&mut self) {
        if !self.entries.is_empty() {
            self.cursor = (self.cursor + 1) % self.entries.len();
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

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
            queue.put_command(CM_OPEN_FILE, Some(Box::new(req)));
        }
    }

    fn poll_results(&mut self) {
        let SearchState::Searching(rx) = &self.search else {
            return;
        };
        let mut got = false;
        loop {
            match rx.try_recv() {
                Ok(batch) => {
                    self.entries.extend(batch);
                    got = true;
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.search = if self.entries.is_empty() {
                        SearchState::NoMatches
                    } else {
                        SearchState::Done
                    };
                    got = true;
                    break;
                }
            }
        }
        if got {
            self.state.mark_dirty();
        }
    }

    fn status_line(&self) -> String {
        match &self.search {
            SearchState::Searching(_) => {
                format!("⟳ Searching... ({} found)", self.entries.len())
            }
            SearchState::Done => {
                format!("✓ {} results", self.entries.len())
            }
            SearchState::NoMatches => "✗ No matches".to_string(),
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
            Style {
                bg: Color::Ansi(4),
                attrs: Attrs { underline: true, ..Attrs::default() },
                ..Style::default()
            }
        } else {
            Style { bg: Color::Ansi(8), ..Style::default() }
        };

        // Reserve last row for status
        let content_h = b.h.saturating_sub(1) as usize;

        for row in 0..content_h {
            let idx = self.scroll + row;
            let y = b.y + row as u16;
            if idx >= self.entries.len() {
                surface.hline(b.x, y, b.w, ' ', normal);
                continue;
            }
            let entry = &self.entries[idx];
            let style = if idx == self.cursor { cursor_style } else { normal };
            surface.hline(b.x, y, b.w, ' ', style);

            let path_str = entry.path.strip_prefix(&self.root)
                .unwrap_or(&entry.path)
                .to_string_lossy();
            let loc = format!("{}:{}:", path_str, entry.line + 1);
            surface.print(b.x, y, &loc, if idx == self.cursor { style } else { dim });
            let text_x = b.x + loc.len().min(b.w as usize) as u16;
            if text_x < b.x + b.w {
                surface.print(text_x, y, &entry.text, style);
            }
        }

        // Status line at bottom
        let status_y = b.y + b.h - 1;
        let status = self.status_line();
        let status_style = match &self.search {
            SearchState::Searching(_) => Style { fg: Color::Ansi(11), ..Style::default() },
            SearchState::Done => Style { fg: Color::Ansi(10), ..Style::default() },
            SearchState::NoMatches => Style { fg: Color::Ansi(9), ..Style::default() },
        };
        surface.hline(b.x, status_y, b.w, ' ', status_style);
        surface.print(b.x, status_y, &status, status_style);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Tick = event {
            self.poll_results();
            return HandleResult::Ignored;
        }
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
            KeyCode::Right => {
                self.open_current(queue);
                queue.put_command(crate::commands::CM_FOCUS_CENTER, None);
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
