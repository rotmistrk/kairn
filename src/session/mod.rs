//! Session save/restore.
//! Auto-state: $PWD/.kairn.state (auto-save on quit, auto-restore on launch)
//! Named sessions: ~/.kairn/sessions/<name>.json (explicit save/load)

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::layout::{LayoutMode, PanelSizes};
use crate::tab::Tab;

/// Serializable snapshot of the full app state.
#[derive(Debug, Serialize, Deserialize)]
pub struct Session {
    pub name: String,
    pub workspace_root: String,
    pub layout_mode: LayoutMode,
    pub panel_sizes: PanelSizes,
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
    pub open_file: Option<String>,
}

// ── Auto-state ($PWD/.kairn.state) ──────────

fn state_path(workspace: &Path) -> PathBuf {
    workspace.join(".kairn.state")
}

pub fn auto_save(workspace: &Path, session: &Session) -> Result<()> {
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(state_path(workspace), json)?;
    Ok(())
}

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

pub fn save(session: &Session) -> Result<()> {
    let path = sessions_dir()?.join(format!("{}.json", session.name));
    let json = serde_json::to_string_pretty(session)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load(name: &str) -> Result<Session> {
    let path = sessions_dir()?.join(format!("{name}.json"));
    let json = std::fs::read_to_string(path)?;
    Ok(serde_json::from_str(&json)?)
}

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

/// Query `kiro-cli chat --list-sessions` and return session UUIDs (most recent first).
pub fn list_kiro_sessions(kiro_cmd: &str) -> Vec<String> {
    let output = std::process::Command::new(kiro_cmd)
        .args(["chat", "--list-sessions"])
        .output();
    let output = match output {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    // Lines look like: "Chat SessionId: <ansi>UUID<ansi>"
    // Strip ANSI and extract UUIDs after "Chat SessionId: "
    let ansi_re = match regex::Regex::new(r"\x1b\[[0-9;]*m") {
        Ok(r) => r,
        Err(_) => return Vec::new(),
    };
    let uuid_re = match regex::Regex::new(
        r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}",
    ) {
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
