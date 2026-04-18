pub mod kiro;
pub mod shell;

use serde::{Deserialize, Serialize};

use self::kiro::KiroProcess;
use self::shell::run_command;

/// Identifies what kind of tab this is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TabKind {
    Kiro { session_name: String },
    Shell,
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
    Kiro(KiroProcess),
}

/// A tab with its optional live backend.
pub struct LiveTab {
    pub meta: Tab,
    pub backend: Option<Backend>,
    pub scroll: usize,
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
            kind: TabKind::Shell,
            title: "sh".to_string(),
            output: vec!["Type a command and press Enter.".to_string()],
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

    pub fn add_shell_tab(&mut self, _command: String) -> TabId {
        let id = TabId(self.next_id);
        self.next_id += 1;
        let meta = Tab {
            id,
            kind: TabKind::Shell,
            title: "sh".to_string(),
            output: Vec::new(),
        };
        self.tabs.push(LiveTab {
            meta,
            backend: None,
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

    /// Run a command in the active shell tab.
    pub fn run_in_active(&mut self, cmd: &str) {
        let tab = match self.tabs.get_mut(self.active) {
            Some(t) => t,
            None => return,
        };
        tab.meta.output.push(format!("$ {cmd}"));
        match run_command(cmd) {
            Ok(result) => {
                append_output(&mut tab.meta.output, &result.stdout);
                append_output(&mut tab.meta.output, &result.stderr);
                if !result.success {
                    let code = result.code.unwrap_or(-1);
                    tab.meta.output.push(format!("[exit {code}]"));
                }
            }
            Err(e) => {
                tab.meta.output.push(format!("[error: {e}]"));
            }
        }
        if tab.follow {
            tab.scroll = tab.meta.output.len();
        }
    }

    /// Send a line to the active Kiro tab.
    pub fn send_to_active_kiro(&mut self, text: &str) {
        let tab = match self.tabs.get_mut(self.active) {
            Some(t) => t,
            None => return,
        };
        tab.meta.output.push(format!("> {text}"));
        if let Some(Backend::Kiro(kp)) = &mut tab.backend {
            let _ = kp.send_line(text);
        }
        if tab.follow {
            tab.scroll = tab.meta.output.len();
        }
    }

    /// Poll Kiro tabs for new output.
    pub fn poll_output(&mut self) {
        let mut buf = [0u8; 4096];
        for tab in &mut self.tabs {
            let kp = match &mut tab.backend {
                Some(Backend::Kiro(kp)) => kp,
                None => continue,
            };
            let got_output = poll_kiro(kp, &mut buf, &mut tab.meta.output);
            if got_output && tab.follow {
                tab.scroll = tab.meta.output.len();
            }
        }
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

    pub fn active_scroll(&self) -> usize {
        self.tabs.get(self.active).map_or(0, |t| t.scroll)
    }

    pub fn scroll_active(&mut self, delta: isize, viewport_h: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            let max = tab.meta.output.len().saturating_sub(viewport_h);
            let new = (tab.scroll as isize).saturating_add(delta);
            tab.scroll = (new.max(0) as usize).min(max);
            tab.follow = tab.scroll >= max;
        }
    }

    pub fn snap_to_bottom(&mut self, viewport_h: usize) {
        if let Some(tab) = self.tabs.get_mut(self.active) {
            tab.scroll = tab.meta.output.len().saturating_sub(viewport_h);
            tab.follow = true;
        }
    }

    pub fn snapshot(&self) -> (Vec<Tab>, usize) {
        let tabs = self.tabs.iter().map(|t| t.meta.clone()).collect();
        (tabs, self.active)
    }

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

    pub fn tab_labels(&self) -> Vec<(&str, bool)> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(i, t)| (t.meta.title.as_str(), i == self.active))
            .collect()
    }

    /// Is the active tab a shell tab?
    pub fn active_is_shell(&self) -> bool {
        self.tabs
            .get(self.active)
            .is_some_and(|t| matches!(t.meta.kind, TabKind::Shell))
    }
}

fn append_output(output: &mut Vec<String>, text: &str) {
    for line in text.lines() {
        output.push(line.to_string());
    }
}

fn poll_kiro(kp: &mut KiroProcess, buf: &mut [u8], output: &mut Vec<String>) -> bool {
    let mut got = false;

    let n = kp.try_read_stdout(buf);
    if n > 0 {
        let text = String::from_utf8_lossy(&buf[..n]);
        let clean = strip_ansi(&text);
        for line in clean.lines() {
            if !line.is_empty() {
                output.push(line.to_string());
            }
        }
        got = true;
    }

    let n = kp.try_read_stderr(buf);
    if n > 0 {
        let text = String::from_utf8_lossy(&buf[..n]);
        let clean = strip_ansi(&text);
        for line in clean.lines() {
            if !line.is_empty() {
                // Prefix stderr lines so they can be styled differently
                output.push(format!("⚠ {line}"));
            }
        }
        got = true;
    }

    got
}

/// Strip ANSI escape sequences and control characters.
fn strip_ansi(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            0x1b => i = skip_esc(bytes, i),
            b'\n' | b'\t' => {
                out.push(bytes[i] as char);
                i += 1;
            }
            0x00..=0x1f | 0x7f => i += 1,
            _ => {
                if is_esc_fragment(bytes, i) {
                    i = skip_fragment(bytes, i);
                } else {
                    out.push(bytes[i] as char);
                    i += 1;
                }
            }
        }
    }
    out
}

fn skip_esc(bytes: &[u8], start: usize) -> usize {
    let mut i = start + 1;
    if i >= bytes.len() {
        return i;
    }
    match bytes[i] {
        b'[' => {
            i += 1;
            while i < bytes.len() {
                if bytes[i].is_ascii_alphabetic() || bytes[i] == b'~' {
                    return i + 1;
                }
                i += 1;
            }
            i
        }
        b']' => {
            i += 1;
            while i < bytes.len() {
                if bytes[i] == 0x07 || bytes[i] == b'\\' {
                    return i + 1;
                }
                i += 1;
            }
            i
        }
        b'(' | b')' => i + 2,
        _ => i + 1,
    }
}

fn is_esc_fragment(bytes: &[u8], pos: usize) -> bool {
    let mut i = pos;
    if i < bytes.len() && bytes[i] == b';' {
        i += 1;
    }
    let start = i;
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b';') {
        i += 1;
    }
    if i == start {
        return false;
    }
    i < bytes.len()
        && matches!(
            bytes[i],
            b'm' | b'h' | b'l' | b'G' | b'B' | b'A' | b'C' | b'D' | b'H' | b'J' | b'K'
        )
}

fn skip_fragment(bytes: &[u8], pos: usize) -> usize {
    let mut i = pos;
    if i < bytes.len() && bytes[i] == b';' {
        i += 1;
    }
    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b';') {
        i += 1;
    }
    if i < bytes.len() && bytes[i].is_ascii_alphabetic() {
        i + 1
    } else {
        i
    }
}
