//! ResultsView — quickfix-style list for LSP refs, grep, build errors.
//!
//! Opens in the tool panel. Enter opens file (keeps focus), Right opens + moves focus.

use std::path::{Path, PathBuf};

use txv_core::cell::Style;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;
use txv_widgets::tiled_workspace::commands::CM_TW_FOCUS_PANEL;

use crate::commands::{OpenFileRequest, CM_OPEN_FILE, CM_TAB_CLOSE};

pub use super::result_entry::ResultEntry;

/// Quickfix-style results list view.
pub struct ResultsView {
    state: ViewState,
    entries: Vec<ResultEntry>,
    cursor: usize,
    scroll: usize,
    title: String,
    root: PathBuf,
    done: bool,
}

impl ResultsView {
    pub fn new(title: &str, entries: Vec<ResultEntry>) -> Self {
        Self {
            state: ViewState::default(),
            entries,
            cursor: 0,
            scroll: 0,
            title: title.to_string(),
            root: PathBuf::new(),
            done: true,
        }
    }

    /// Create an empty view in searching state.
    pub fn searching(title: &str, root: &Path) -> Self {
        Self {
            state: ViewState::default(),
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            title: title.to_string(),
            root: root.to_path_buf(),
            done: false,
        }
    }

    pub fn with_root(mut self, root: &Path) -> Self {
        self.root = root.to_path_buf();
        self
    }

    /// Append entries from async grep. Mark done when search completes.
    pub fn append(&mut self, entries: Vec<ResultEntry>, done: bool) {
        self.entries.extend(entries);
        self.done = done;
        self.state.mark_dirty();
    }
    pub fn current_entry(&self) -> Option<&ResultEntry> {
        self.entries.get(self.cursor)
    }

    pub fn entries(&self) -> &[ResultEntry] {
        &self.entries
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
        let h = self.state.bounds().h.saturating_sub(1) as usize;
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
    }

    fn open_current(&self) {
        if let Some(entry) = self.entries.get(self.cursor) {
            let req = OpenFileRequest::at(entry.path.clone(), entry.line, entry.col);
            self.state.put_command(CM_OPEN_FILE, Some(Box::new(req)));
        }
    }

    fn draw_entries(&mut self, w: u16, h: u16, dim: Style, normal: Style, cursor_style: Style) {
        let content_h = h.saturating_sub(1) as usize;
        for row in 0..content_h {
            let idx = self.scroll + row;
            let y = row as u16;
            if idx >= self.entries.len() {
                self.state.buffer_mut().hline(0, y, w, ' ', normal);
                continue;
            }
            let entry = &self.entries[idx];
            let style = if idx == self.cursor {
                cursor_style
            } else {
                normal
            };
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            let path_str = entry
                .path
                .strip_prefix(&self.root)
                .unwrap_or(&entry.path)
                .to_string_lossy();
            let loc = format!("{}:{}:", path_str, entry.line + 1);
            self.state.buffer_mut().print(
                0,
                y,
                &loc,
                if idx == self.cursor {
                    style
                } else {
                    dim
                },
            );
            let text_x = loc.len().min(w as usize) as u16;
            if text_x < w {
                self.state.buffer_mut().print(text_x, y, &entry.text, style);
            }
        }
    }

    fn draw_status_line(&mut self, w: u16, h: u16) {
        let pal = palette();
        let status_y = h - 1;
        let status = if !self.done {
            format!("⟳ Searching... ({} found)", self.entries.len())
        } else if self.entries.is_empty() {
            "✗ No matches".to_string()
        } else {
            format!("✓ {} results", self.entries.len())
        };
        let status_style = if !self.done {
            pal.style(StyleId::StateWarning)
        } else if self.entries.is_empty() {
            pal.style(StyleId::StateError)
        } else {
            pal.style(StyleId::StateSuccess)
        };
        self.state.buffer_mut().hline(0, status_y, w, ' ', status_style);
        self.state.buffer_mut().print(0, status_y, &status, status_style);
    }
}

impl View for ResultsView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        &self.title
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = palette();
        let dim = pal.style(StyleId::Dim);
        let normal = Style::default();
        let cursor_style = if self.state.is_focused() {
            pal.style(StyleId::CursorFocused)
        } else {
            pal.style(StyleId::CursorUnfocused)
        };
        self.draw_entries(w, h, dim, normal, cursor_style);
        self.draw_status_line(w, h);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        log::error!("RESULTS_HANDLE event={:?}", std::mem::discriminant(event));
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
                self.open_current();
                HandleResult::Consumed
            }
            KeyCode::Right => {
                self.open_current();
                self.state.put_command(CM_TW_FOCUS_PANEL, Some(Box::new(1usize)));
                HandleResult::Consumed
            }
            KeyCode::Char('n') => {
                self.next();
                self.open_current();
                HandleResult::Consumed
            }
            KeyCode::Char('p') => {
                self.prev();
                self.open_current();
                HandleResult::Consumed
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.state.put_command(CM_TAB_CLOSE, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}
