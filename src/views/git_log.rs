//! GitLogView — scrollable commit history in the tool panel.

use std::collections::HashMap;
use std::path::PathBuf;

use txv_core::cell::Color;
use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::commands::{CM_GIT_BASE_CHANGED, CM_GIT_SET_BASE, CM_TAB_CLOSE};
use crate::git_log::{CommitEntry, LogState, SharedLog};

/// Scrollable git commit log view.
pub struct GitLogView {
    pub(super) state: ViewState,
    pub(super) shared: SharedLog,
    pub(super) entries: Vec<CommitEntry>,
    pub(super) cursor: usize,
    pub(super) scroll: usize,
    pub(super) done: bool,
    /// Currently active diff base hashes per root.
    pub(super) current_base: HashMap<PathBuf, String>,
}

impl GitLogView {
    pub fn new(shared: SharedLog) -> Self {
        Self {
            state: ViewState::default(),
            shared,
            entries: Vec::new(),
            cursor: 0,
            scroll: 0,
            done: false,
            current_base: HashMap::new(),
        }
    }

    /// Set the current diff base map (shown with indicators).
    pub fn set_current_base(&mut self, base: HashMap<PathBuf, String>) {
        self.current_base = base;
        self.state.mark_dirty();
    }

    fn sync_from_shared(&mut self) {
        if self.done {
            return;
        }
        let Ok(guard) = self.shared.lock() else {
            return;
        };
        match &*guard {
            LogState::Loading => {}
            LogState::Ready(entries) => {
                self.entries = entries.clone();
                self.done = true;
                self.state.mark_dirty();
            }
            LogState::Error(e) => {
                self.entries = vec![CommitEntry {
                    hash: String::new(),
                    summary: e.clone(),
                    author: String::new(),
                    time_secs: 0,
                    decorations: Vec::new(),
                    root: PathBuf::new(),
                    root_color: Color::Reset,
                }];
                self.done = true;
                self.state.mark_dirty();
            }
        }
    }

    fn sync_scroll(&mut self) {
        let h = self.visible_height();
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
    }

    fn visible_height(&self) -> usize {
        self.state.bounds().h() as usize
    }

    pub(super) fn is_base_row(&self, entry: &CommitEntry) -> bool {
        self.current_base.get(&entry.root).is_some_and(|b| *b == entry.hash)
    }
}

impl View for GitLogView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        "Log"
    }

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let pal = palette();
        let dim = pal.style(StyleId::Dim);

        if !self.done {
            self.state.buffer_mut().print(0, 0, "Loading...", dim);
            return;
        }

        self.draw_rows(w, h);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            self.sync_from_shared();
            return HandleResult::Ignored;
        }
        if let Event::Command {
            id,
            data,
            broadcast: true,
        } = event
        {
            if *id == CM_GIT_BASE_CHANGED {
                if let Some(map) = data.as_ref().and_then(|d| d.downcast_ref::<HashMap<PathBuf, String>>()) {
                    self.current_base = map.clone();
                    self.state.mark_dirty();
                }
                return HandleResult::Consumed;
            }
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        self.handle_key(key)
    }
}

impl GitLogView {
    fn handle_key(&mut self, key: &KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Char('j') | KeyCode::Down => {
                if self.cursor + 1 < self.entries.len() {
                    self.cursor += 1;
                    self.sync_scroll();
                    self.state.mark_dirty();
                }
                HandleResult::Consumed
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.cursor = self.cursor.saturating_sub(1);
                self.sync_scroll();
                self.state.mark_dirty();
                HandleResult::Consumed
            }
            KeyCode::Char('q') => {
                self.state.put_command(CM_TAB_CLOSE, None);
                HandleResult::Consumed
            }
            KeyCode::Char('b') => {
                self.handle_set_base();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn handle_set_base(&mut self) {
        if let Some(entry) = self.entries.get(self.cursor).cloned() {
            let is_current = self.current_base.get(&entry.root).is_some_and(|b| *b == entry.hash);
            let payload: Option<(PathBuf, String)> = if is_current {
                self.current_base.remove(&entry.root);
                None
            } else {
                self.current_base.insert(entry.root.clone(), entry.hash.clone());
                Some((entry.root, entry.hash))
            };
            self.state.put_command(CM_GIT_SET_BASE, Some(Box::new(payload)));
            self.state.mark_dirty();
        }
    }
}
