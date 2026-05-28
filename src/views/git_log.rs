//! GitLogView — scrollable commit history in the tool panel.

use txv_core::cell::Style;
use txv_core::prelude::*;

use crate::git_log::{CommitEntry, LogState, SharedLog};

/// Scrollable git commit log view.
pub struct GitLogView {
    state: ViewState,
    shared: SharedLog,
    entries: Vec<CommitEntry>,
    cursor: usize,
    scroll: usize,
    done: bool,
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
        }
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
        self.state.bounds().h as usize
    }

    fn format_entry(e: &CommitEntry, width: usize) -> String {
        let decor = if e.decorations.is_empty() {
            String::new()
        } else {
            format!(" ({})", e.decorations.join(", "))
        };
        let age = format_relative_time(e.time_secs);
        let prefix = format!("* {}{} {}", e.hash, decor, e.summary);
        let suffix = format!("{} {}", e.author, age);
        let gap = width.saturating_sub(prefix.len() + suffix.len() + 1);
        if gap > 0 {
            format!("{}{:>pad$} {}", prefix, "", suffix, pad = gap)
        } else {
            let max_prefix = width.saturating_sub(suffix.len() + 2);
            format!("{:.w$} {}", prefix, suffix, w = max_prefix)
        }
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
        let pal = txv_core::palette::palette();
        let normal = Style::default();
        let dim = pal.style(txv_core::palette::StyleId::Dim);
        let cursor_style = if self.state.is_focused() {
            pal.style(txv_core::palette::StyleId::CursorFocused)
        } else {
            pal.style(txv_core::palette::StyleId::CursorUnfocused)
        };

        if !self.done {
            self.state.buffer_mut().print(0, 0, "Loading...", dim);
            return;
        }

        let rows = h as usize;
        for row in 0..rows {
            let idx = self.scroll + row;
            let y = row as u16;
            if idx >= self.entries.len() {
                self.state.buffer_mut().hline(0, y, w, ' ', normal);
                continue;
            }
            let style = if idx == self.cursor {
                cursor_style
            } else {
                normal
            };
            let line = Self::format_entry(&self.entries[idx], w as usize);
            self.state.buffer_mut().hline(0, y, w, ' ', style);
            self.state.buffer_mut().print(0, y, &line, style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Tick = event {
            self.sync_from_shared();
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        match key.code {
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
                self.state.put_command(crate::commands::CM_TAB_CLOSE, None);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }
}

fn format_relative_time(epoch_secs: i64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    let diff = (now - epoch_secs).max(0);
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else if diff < 604800 {
        format!("{}d ago", diff / 86400)
    } else if diff < 2592000 {
        format!("{}w ago", diff / 604800)
    } else {
        format!("{}mo ago", diff / 2592000)
    }
}
