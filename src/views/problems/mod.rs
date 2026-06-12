//! ProblemsView — live LSP diagnostics panel.
//!
//! Shows all diagnostics across open files, updates on CM_DIAGNOSTIC.
//! Enter opens file at diagnostic location.

mod draw;

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::prelude::*;
use txv_widgets::tiled_workspace::commands::CM_TW_FOCUS_PANEL;

use crate::commands::{OpenFileRequest, CM_OPEN_FILE, CM_TAB_CLOSE};
use crate::lsp::diagnostics::{Diagnostic, Severity};

use draw::Entry;

pub struct ProblemsView {
    state: ViewState,
    /// Diagnostics keyed by file URI.
    store: HashMap<String, Vec<Diagnostic>>,
    /// Flattened entries for display.
    entries: Vec<Entry>,
    cursor: usize,
    scroll: usize,
    root: PathBuf,
}

impl ProblemsView {
    pub fn new(root: &std::path::Path) -> Self {
        Self {
            state: ViewState::default(),
            store: HashMap::new(),
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            root: root.to_path_buf(),
        }
    }

    /// Update diagnostics for a file URI. Called by the handler.
    pub fn update_diagnostics(&mut self, uri: &str, diags: Vec<Diagnostic>) {
        if diags.is_empty() {
            self.store.remove(uri);
        } else {
            self.store.insert(uri.to_string(), diags);
        }
        self.rebuild_entries();
    }

    fn rebuild_entries(&mut self) {
        self.entries.clear();
        let mut uris: Vec<_> = self.store.keys().cloned().collect();
        uris.sort();
        for uri in &uris {
            let path = PathBuf::from(uri);
            if let Some(diags) = self.store.get(uri) {
                for d in diags {
                    self.entries.push(Entry {
                        path: path.clone(),
                        line: d.line,
                        severity: d.severity,
                        message: d.message.clone(),
                    });
                }
            }
        }
        if self.cursor > self.entries.len().saturating_sub(1) {
            self.cursor = self.entries.len().saturating_sub(1);
        }
        self.state.mark_dirty();
    }

    fn open_at_cursor(&mut self) {
        let Some(entry) = self.entries.get(self.cursor) else {
            return;
        };
        let req = OpenFileRequest {
            path: entry.path.clone(),
            line: Some(entry.line as u32),
            col: None,
            diff: false,
        };
        self.state.put_command(CM_OPEN_FILE, Some(Box::new(req)));
    }

    fn open_and_focus(&mut self) {
        self.open_at_cursor();
        self.state.put_command(CM_TW_FOCUS_PANEL, Some(Box::new(1usize)));
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    pub fn error_count(&self) -> usize {
        self.entries.iter().filter(|e| e.severity == Severity::Error).count()
    }

    /// Format all diagnostics as text for MCP access.
    pub fn format_for_mcp(&self) -> String {
        self.entries
            .iter()
            .map(|e| {
                let sev = match e.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                    Severity::Hint => "hint",
                };
                let rel = e.path.strip_prefix(&self.root).unwrap_or(&e.path);
                format!("{}:{}:{}: {}", rel.display(), e.line + 1, sev, e.message)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl View for ProblemsView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        "Problems"
    }

    fn draw(&mut self) {
        draw::draw(self);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Key(key) => self.handle_key(key),
            _ => HandleResult::Ignored,
        }
    }
}

impl ProblemsView {
    fn handle_key(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        if !self.state.is_focused() {
            return HandleResult::Ignored;
        }
        match key.code() {
            KeyCode::Char('j') | KeyCode::Down => self.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.move_up(),
            KeyCode::Enter => self.open_at_cursor(),
            KeyCode::Right => self.open_and_focus(),
            KeyCode::Char('n') => {
                self.move_down();
                self.open_at_cursor();
            }
            KeyCode::Char('p') => {
                self.move_up();
                self.open_at_cursor();
            }
            KeyCode::Char('g') | KeyCode::Home => {
                self.cursor = 0;
                self.state.mark_dirty();
            }
            KeyCode::Char('G') | KeyCode::End => {
                self.cursor = self.entries.len().saturating_sub(1);
                self.state.mark_dirty();
            }
            KeyCode::Char('q') => self.state.put_command(CM_TAB_CLOSE, None),
            _ => return HandleResult::Ignored,
        }
        HandleResult::Consumed
    }

    fn move_down(&mut self) {
        if self.cursor + 1 < self.entries.len() {
            self.cursor += 1;
            self.state.mark_dirty();
        }
    }

    fn move_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.state.mark_dirty();
        }
    }
}
