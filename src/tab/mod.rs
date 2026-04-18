pub mod kiro;
pub mod shell;

use serde::{Deserialize, Serialize};

use self::kiro::KiroProcess;
use self::shell::PtyShell;

/// Identifies what kind of tab this is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabKind {
    Kiro { session_name: String },
    Shell { command: String },
}

/// Serializable tab metadata (for session persistence).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tab {
    pub id: TabId,
    pub kind: TabKind,
    pub title: String,
    pub output: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TabId(pub u32);

/// A live backend attached to a tab.
pub enum Backend {
    Shell(PtyShell),
    Kiro(KiroProcess),
}

/// A tab with its optional live backend.
pub struct LiveTab {
    pub meta: Tab,
    pub backend: Option<Backend>,
    pub scroll: usize,
    /// When true, auto-scroll to bottom on new output.
    pub follow: bool,
}

/// Manages open tabs and their backends.
pub struct TabManager {
    tabs: Vec<LiveTab>,
    active: usize,
    next_id: u32,
}

impl Default for TabManager {
    fn default() -> Self {
        let meta = Tab {
            id: TabId(0),
            kind: TabKind::Kiro {
                session_name: "main".to_string(),
            },
            title: "kiro:main".to_string(),
            output: vec!["Welcome to kairn.".to_string()],
        };
        Self {
            tabs: vec![LiveTab {
                meta,
                backend: None,
                scroll: 0,
                follow: true,
            }],
            active: 0,
            next_id: 1,
        }
    }
}

impl TabManager {
    pub fn active_title(&self) -> &str {
        self.tabs
            .get(self.active)
            .map(|t| t.meta.title.as_str())
            .unwrap_or("no tabs")
    }

    pub fn active_content(&self) -> String {
        self.tabs
            .get(self.active)
            .map(|t| t.meta.output.join("\n"))
            .unwrap_or_default()
    }

    pub fn add_shell_tab(&mut self, command: String) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let backend = PtyShell::spawn(&command, 80, 24).ok().map(Backend::Shell);
        let meta = Tab {
            id,
            kind: TabKind::Shell {
                command: command.clone(),
            },
            title: format!("sh:{command}"),
            output: Vec::new(),
        };
        self.tabs.push(LiveTab {
            meta,
            backend,
            scroll: 0,
            follow: true,
        });
        self.active = self.tabs.len() - 1;
        id
    }

    pub fn add_kiro_tab(&mut self, session_name: String) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let backend = KiroProcess::spawn("kiro-cli").ok().map(Backend::Kiro);
        let meta = Tab {
            id,
            kind: TabKind::Kiro {
                session_name: session_name.clone(),
            },
            title: format!("kiro:{session_name}"),
            output: Vec::new(),
        };
        self.tabs.push(LiveTab {
            meta,
            backend,
            scroll: 0,
            follow: true,
        });
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

    pub fn push_to_active(&mut self, line: String) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.meta.output.push(line);
        }
    }

    /// Write raw bytes to the active tab's backend.
    pub fn write_to_active(&mut self, data: &[u8]) {
        let tab = match self.tabs.get_mut(self.active) {
            Some(t) => t,
            None => return,
        };
        match &mut tab.backend {
            Some(Backend::Shell(pty)) => {
                let _ = pty.write_all(data);
            }
            Some(Backend::Kiro(kp)) => {
                write_to_kiro(kp, data);
            }
            None => {}
        }
    }

    /// Poll all tabs for new output. Call from event loop.
    pub fn poll_output(&mut self) {
        let mut buf = [0u8; 4096];
        for tab in &mut self.tabs {
            let n = read_backend(&mut tab.backend, &mut buf);
            if n == 0 {
                continue;
            }
            let text = String::from_utf8_lossy(&buf[..n]);
            let clean = strip_ansi(&text);
            for line in clean.lines() {
                if !line.is_empty() {
                    tab.meta.output.push(line.to_string());
                }
            }
            if tab.follow {
                tab.scroll = tab.meta.output.len();
            }
        }
    }

    /// Resize the active shell PTY.
    pub fn resize_active(&mut self, cols: u16, rows: u16) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            if let Some(Backend::Shell(pty)) = &tab.backend {
                let _ = pty.resize(cols, rows);
            }
        }
    }

    /// Snapshot tab metadata for session persistence.
    pub fn snapshot(&self) -> (Vec<Tab>, usize) {
        let tabs = self.tabs.iter().map(|t| t.meta.clone()).collect();
        (tabs, self.active)
    }

    /// Restore tabs from saved session (no live backends).
    pub fn restore(&mut self, tabs: Vec<Tab>, active: usize) {
        let max_id = tabs.iter().map(|t| t.id.0).max().unwrap_or(0);
        self.tabs = tabs
            .into_iter()
            .map(|meta| LiveTab {
                meta,
                backend: None,
                scroll: 0,
                follow: true,
            })
            .collect();
        self.active = active.min(self.tabs.len().saturating_sub(1));
        self.next_id = max_id + 1;
    }

    /// Tab bar labels for rendering.
    pub fn tab_labels(&self) -> Vec<(&str, bool)> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(i, t)| (t.meta.title.as_str(), i == self.active))
            .collect()
    }

    /// Current scroll offset of the active tab.
    pub fn active_scroll(&self) -> usize {
        self.tabs.get(self.active).map_or(0, |t| t.scroll)
    }

    /// Scroll the active tab by delta lines.
    pub fn scroll_active(&mut self, delta: isize, viewport_h: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            let max = tab.meta.output.len().saturating_sub(viewport_h);
            let new = (tab.scroll as isize).saturating_add(delta);
            tab.scroll = (new.max(0) as usize).min(max);
            // If scrolled away from bottom, stop following
            tab.follow = tab.scroll >= max;
        }
    }

    /// Snap active tab to bottom.
    pub fn snap_to_bottom(&mut self, viewport_h: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.scroll = tab.meta.output.len().saturating_sub(viewport_h);
            tab.follow = true;
        }
    }
}

fn write_to_kiro(kp: &mut KiroProcess, data: &[u8]) {
    if let Ok(s) = std::str::from_utf8(data) {
        let trimmed = s.trim_end();
        if !trimmed.is_empty() {
            let _ = kp.send_line(trimmed);
        }
    }
}

fn read_backend(backend: &mut Option<Backend>, buf: &mut [u8]) -> usize {
    match backend {
        Some(Backend::Shell(pty)) => pty.try_read(buf).unwrap_or(0),
        Some(Backend::Kiro(kp)) => kp.try_read(buf).unwrap_or(0),
        None => 0,
    }
}

/// Strip ANSI escape sequences and carriage returns from terminal output.
fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\x1b' => skip_escape_seq(&mut chars),
            '\r' => {}
            _ => out.push(c),
        }
    }
    out
}

fn skip_escape_seq(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    match chars.peek() {
        Some('[') => {
            chars.next();
            consume_until_alpha(chars);
        }
        Some(']') => {
            chars.next();
            consume_until_bel_or_st(chars);
        }
        _ => {
            chars.next();
        }
    }
}

fn consume_until_alpha(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    for c in chars.by_ref() {
        if c.is_ascii_alphabetic() || c == '~' {
            break;
        }
    }
}

fn consume_until_bel_or_st(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
    for c in chars.by_ref() {
        if c == '\x07' || c == '\\' {
            break;
        }
    }
}
