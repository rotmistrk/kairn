//! Session save/restore.
//!
//! Auto-state: `$PWD/.kairn.state` (auto-save on quit, auto-restore on launch).
//! Named sessions: `~/.kairn/sessions/<name>.json` (explicit save/load).

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::layout::{LayoutMode, PanelSizes};
use crate::tab::Tab;

/// Serializable snapshot of the full app state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    /// Session name (for named sessions).
    pub name: String,
    /// Workspace root path.
    pub workspace_root: String,
    /// Current layout mode.
    pub layout_mode: LayoutMode,
    /// Panel sizing state.
    pub panel_sizes: PanelSizes,
    /// Open terminal tabs.
    pub tabs: Vec<Tab>,
    /// Index of the active tab.
    pub active_tab: usize,
    /// Currently open file in the main panel.
    pub open_file: Option<String>,
}

/// Extended session data for the f4 architecture.
///
/// Stored alongside the base [`Session`] when available.
/// Backward-compatible: missing fields use defaults on load.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SessionExtras {
    /// Open buffer paths.
    #[serde(default)]
    pub open_buffers: Vec<String>,
    /// Active buffer index.
    #[serde(default)]
    pub active_buffer: usize,
    /// Cursor positions per buffer: (line, col).
    #[serde(default)]
    pub cursor_positions: Vec<(usize, usize)>,
    /// Active keymap name.
    #[serde(default)]
    pub keymap: Option<String>,
}

// ── Auto-state ($PWD/.kairn.state) ──────────

fn state_path(workspace: &Path) -> PathBuf {
    workspace.join(".kairn.state")
}

/// Save session state to `$PWD/.kairn.state`.
pub fn auto_save(workspace: &Path, session: &Session) -> Result<()> {
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(state_path(workspace), json)?;
    Ok(())
}

/// Load session state from `$PWD/.kairn.state`.
pub fn auto_load(workspace: &Path) -> Result<Option<Session>> {
    let path = state_path(workspace);
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(path)?;
    let session: Session = serde_json::from_str(&json)?;
    Ok(Some(session))
}

// ── Named sessions (~/.kairn/sessions/) ─────

fn sessions_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|e| anyhow::anyhow!("HOME not set: {e}"))?;
    let dir = home.join(".kairn").join("sessions");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Save a named session.
pub fn save(session: &Session) -> Result<()> {
    let path = sessions_dir()?.join(format!("{}.json", session.name));
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(path, json)?;
    Ok(())
}

/// Load a named session.
pub fn load(name: &str) -> Result<Session> {
    let path = sessions_dir()?.join(format!("{name}.json"));
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

/// List all named sessions.
pub fn list_sessions() -> Result<Vec<String>> {
    let dir = sessions_dir()?;
    let mut names = Vec::new();
    for entry in std::fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            if let Some(stem) = path.file_stem() {
                names.push(stem.to_string_lossy().to_string());
            }
        }
    }
    names.sort();
    Ok(names)
}

/// Query `kiro-cli chat --list-sessions` and return session UUIDs.
pub fn list_kiro_sessions(kiro_cmd: &str) -> Vec<String> {
    let output = match std::process::Command::new(kiro_cmd)
        .args(["chat", "--list-sessions"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let ansi_re = match regex::Regex::new(r"\x1b\[[0-9;]*m") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let uuid_re =
        match regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}") {
            Ok(r) => r,
            Err(_) => return Vec::new(),
        };
    let mut ids = Vec::new();
    for line in text.lines() {
        let clean = ansi_re.replace_all(line, "");
        if clean.contains("Chat SessionId:") {
            if let Some(m) = uuid_re.find(&clean) {
                ids.push(m.as_str().to_string());
            }
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_session(name: &str) -> Session {
        Session {
            name: name.to_string(),
            workspace_root: "/tmp/test".to_string(),
            layout_mode: LayoutMode::default(),
            panel_sizes: PanelSizes::default(),
            tabs: Vec::new(),
            active_tab: 0,
            open_file: None,
        }
    }

    #[test]
    fn auto_save_and_load() {
        let dir = TempDir::new().unwrap();
        let session = make_session("test");
        auto_save(dir.path(), &session).unwrap();
        let loaded = auto_load(dir.path()).unwrap().unwrap();
        assert_eq!(loaded.name, "test");
    }

    #[test]
    fn auto_load_missing_returns_none() {
        let dir = TempDir::new().unwrap();
        let loaded = auto_load(dir.path()).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn session_roundtrip_json() {
        let session = make_session("roundtrip");
        let json = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "roundtrip");
    }

    #[test]
    fn session_extras_defaults() {
        let extras = SessionExtras::default();
        assert!(extras.open_buffers.is_empty());
        assert!(extras.keymap.is_none());
    }
}
