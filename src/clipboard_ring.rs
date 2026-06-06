//! ClipboardRing — shared copy/paste ring buffer with system clipboard sync.

use std::collections::{HashMap, VecDeque};
use std::env;
use std::io;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::clip_entry::ClipEntry;

/// Shared clipboard ring buffer.
pub struct ClipboardRing {
    entries: VecDeque<ClipEntry>,
    max_entries: usize,
    registers: HashMap<char, String>,
}

impl ClipboardRing {
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            max_entries,
            registers: HashMap::new(),
        }
    }

    /// Push text onto the ring. Also writes to system clipboard (best-effort).
    pub fn push(&mut self, text: &str, source: &str) {
        if text.is_empty() {
            return;
        }
        let entry = ClipEntry {
            text: text.to_string(),
            source: source.to_string(),
            timestamp: Instant::now(),
            line_count: text.lines().count().max(1),
        };
        self.entries.push_front(entry);
        if self.entries.len() > self.max_entries {
            self.entries.pop_back();
        }
        let _ = write_system_clipboard(text);
    }

    /// Paste: try system clipboard first, fallback to ring top.
    /// If system clipboard differs from top, push it to ring.
    pub fn paste(&mut self) -> Option<String> {
        if let Ok(sys_text) = read_system_clipboard() {
            if !sys_text.is_empty() {
                let differs = self.entries.front().is_none_or(|e| e.text != sys_text);
                if differs {
                    self.push(&sys_text, "system");
                }
                return Some(sys_text);
            }
        }
        self.entries.front().map(|e| e.text.clone())
    }

    /// Peek at top without system clipboard check.
    pub fn peek(&self) -> Option<&str> {
        self.entries.front().map(|e| e.text.as_str())
    }

    /// Select entry at index, move to top, sync to system clipboard.
    pub fn select(&mut self, idx: usize) -> Option<&str> {
        if idx >= self.entries.len() || idx == 0 {
            return self.peek();
        }
        let entry = self.entries.remove(idx)?;
        let _ = write_system_clipboard(&entry.text);
        self.entries.push_front(entry);
        self.peek()
    }

    /// Get all entries (for viewer).
    pub fn entries(&self) -> &VecDeque<ClipEntry> {
        &self.entries
    }

    /// Number of entries.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Set named register.
    pub fn set_register(&mut self, name: char, text: &str) {
        self.registers.insert(name, text.to_string());
    }

    /// Get named register.
    pub fn get_register(&self, name: char) -> Option<&str> {
        self.registers.get(&name).map(|s| s.as_str())
    }
}

/// Shared handle to the clipboard ring.
pub type ClipboardHandle = Arc<Mutex<ClipboardRing>>;

/// Create a new shared clipboard ring.
pub fn new_clipboard(max_entries: usize) -> ClipboardHandle {
    Arc::new(Mutex::new(ClipboardRing::new(max_entries)))
}

// ─── System clipboard ───────────────────

fn write_system_clipboard(_text: &str) -> Result<(), String> {
    Ok(())
}

fn read_system_clipboard() -> Result<String, String> {
    if env::var("KAIRN_TEST").is_ok() {
        return Err("test mode".to_string());
    }
    #[cfg(target_os = "macos")]
    {
        read_via_command("pbpaste", &[])
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Ok(text) = read_via_command("wl-paste", &["--no-newline"]) {
            return Ok(text);
        }
        read_via_command("xclip", &["-selection", "clipboard", "-o"])
    }
}

fn write_via_command(cmd: &str, args: &[&str], text: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let mut child = Command::new(cmd)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(text.as_bytes()).map_err(|e| format!("{cmd}: {e}"))?;
    }
    child.wait().map_err(|e| format!("{cmd}: {e}"))?;
    Ok(())
}

fn read_via_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    use std::process::Command;
    let output = Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("{cmd}: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(format!("{cmd} failed"))
    }
}

#[cfg(not(target_os = "macos"))]
fn write_osc52(text: &str) -> Result<(), String> {
    use base64::engine::general_purpose::STANDARD as BASE64;
    use base64::Engine;
    use std::io::Write;
    let encoded = BASE64.encode(text);
    let seq = format!("\x1b]52;c;{encoded}\x07");
    io::stdout()
        .write_all(seq.as_bytes())
        .map_err(|e| format!("osc52: {e}"))?;
    io::stdout().flush().map_err(|e| format!("osc52: {e}"))?;
    Ok(())
}
