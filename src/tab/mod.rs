pub mod shell;

use serde::{Deserialize, Serialize};

use self::shell::PtyTab;

/// Tab kind for serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabKind {
    Kiro {
        session_name: String,
        #[serde(default)]
        session_id: Option<String>,
    },
    Shell,
}

/// Serializable tab metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub kind: TabKind,
    pub title: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub u32);

/// A live tab with its PTY.
pub struct LiveTab {
    pub meta: Tab,
    pub pty: Option<PtyTab>,
}

/// Manages open tabs.
#[derive(Default)]
pub struct TabManager {
    tabs: Vec<LiveTab>,
    active: usize,
    next_id: u32,
}

impl TabManager {
    fn next_ordinal(&self, prefix: &str) -> usize {
        self.tabs
            .iter()
            .filter_map(|t| {
                t.meta
                    .title
                    .strip_prefix(prefix)
                    .and_then(|n| n.parse::<usize>().ok())
            })
            .max()
            .map(|n| n + 1)
            .unwrap_or(1)
    }

    pub fn add_shell_tab(&mut self, cols: u16, rows: u16, cwd: &std::path::Path) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let n = self.next_ordinal("shell:");
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let pty = PtyTab::spawn(&shell, &[], cols, rows, cwd).ok();
        let meta = Tab {
            id,
            kind: TabKind::Shell,
            title: format!("shell:{n}"),
        };
        self.tabs.push(LiveTab { meta, pty });
        self.active = self.tabs.len() - 1;
        id
    }

    pub fn add_kiro_tab(
        &mut self,
        kiro_cmd: &str,
        cols: u16,
        rows: u16,
        cwd: &std::path::Path,
    ) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let n = self.next_ordinal("kiro:");
        let pty = PtyTab::spawn(kiro_cmd, &["chat", "--classic"], cols, rows, cwd).ok();
        let meta = Tab {
            id,
            kind: TabKind::Kiro {
                session_name: format!("{n}"),
                session_id: None,
            },
            title: format!("kiro:{n}"),
        };
        self.tabs.push(LiveTab { meta, pty });
        self.active = self.tabs.len() - 1;
        id
    }

    pub fn close_active(&mut self) {
        if self.tabs.len() > 1 {
            self.tabs.remove(self.active);
            if self.active >= self.tabs.len() {
                self.active = self.tabs.len() - 1;
            }
        }
    }

    pub fn next_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if !self.tabs.is_empty() {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
        }
    }

    /// Write raw bytes to the active tab's PTY.
    pub fn write_to_active(&mut self, data: &[u8]) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            if let Some(pty) = &mut tab.pty {
                pty.write(data);
            }
        }
    }

    /// Poll all tabs for PTY output.
    pub fn poll_output(&mut self) {
        for tab in &mut self.tabs {
            if let Some(pty) = &mut tab.pty {
                pty.poll();
            }
        }
    }

    /// Resize the active tab's PTY.
    pub fn resize_active(&mut self, cols: u16, rows: u16) {
        for tab in &mut self.tabs {
            if let Some(pty) = &mut tab.pty {
                pty.resize(cols, rows);
            }
        }
    }

    /// Get the active tab's termbuf for rendering.
    pub fn active_termbuf(&self) -> Option<&crate::termbuf::TermBuf> {
        self.tabs
            .get(self.active)
            .and_then(|t| t.pty.as_ref())
            .map(|p| &p.termbuf)
    }

    /// Get mutable termbuf for scroll.
    pub fn active_termbuf_mut(&mut self) -> Option<&mut crate::termbuf::TermBuf> {
        self.tabs
            .get_mut(self.active)
            .and_then(|t| t.pty.as_mut())
            .map(|p| &mut p.termbuf)
    }

    pub fn active_title(&self) -> &str {
        self.tabs
            .get(self.active)
            .map(|t| t.meta.title.as_str())
            .unwrap_or("no tabs")
    }

    pub fn tab_labels(&self) -> Vec<(&str, bool)> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(i, t)| (t.meta.title.as_str(), i == self.active))
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    pub fn rename_active(&mut self, name: String) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.meta.title = name;
        }
    }

    /// Snapshot for session persistence.
    pub fn snapshot(&self) -> (Vec<Tab>, usize) {
        let tabs = self.tabs.iter().map(|t| t.meta.clone()).collect();
        (tabs, self.active)
    }

    /// Restore from session (no live PTYs).
    pub fn restore(&mut self, tabs: Vec<Tab>, active: usize) {
        let max_id = tabs.iter().map(|t| t.id.0).max().unwrap_or(0);
        self.tabs = tabs
            .into_iter()
            .map(|meta| LiveTab { meta, pty: None })
            .collect();
        self.active = active.min(self.tabs.len().saturating_sub(1));
        self.next_id = max_id + 1;
    }

    /// Spawn live PTYs for all restored tabs that lack one.
    /// Shell tabs get a fresh shell; Kiro tabs resume via --resume-id
    /// if a session_id is stored, otherwise start fresh.
    pub fn revive_tabs(&mut self, kiro_cmd: &str, cols: u16, rows: u16, cwd: &std::path::Path) {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        for tab in &mut self.tabs {
            if tab.pty.is_some() {
                continue;
            }
            tab.pty = match &tab.meta.kind {
                TabKind::Shell => PtyTab::spawn(&shell, &[], cols, rows, cwd).ok(),
                TabKind::Kiro { session_id, .. } => {
                    let mut args: Vec<&str> = vec!["chat", "--classic"];
                    let id_owned;
                    if let Some(id) = session_id {
                        id_owned = id.clone();
                        args.push("--resume-id");
                        args.push(&id_owned);
                    }
                    PtyTab::spawn(kiro_cmd, &args, cols, rows, cwd).ok()
                }
            };
        }
    }

    /// Update kiro tab session IDs from a list of known IDs (most recent first).
    /// Assigns IDs round-robin to kiro tabs that don't already have one.
    pub fn stamp_kiro_sessions(&mut self, session_ids: &[String]) {
        let mut id_iter = session_ids.iter();
        for tab in &mut self.tabs {
            if let TabKind::Kiro { session_id, .. } = &mut tab.meta.kind {
                if session_id.is_none() {
                    *session_id = id_iter.next().cloned();
                }
            }
        }
    }
}
