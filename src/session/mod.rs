//! Session save/restore.
//! Persists layout, open tabs, open file to ~/.kairn/sessions/.

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

fn sessions_dir() -> Result<std::path::PathBuf> {
    let home = std::env::var("HOME")
        .map(std::path::PathBuf::from)
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
    let session: Session = serde_json::from_str(&json)?;
    Ok(session)
}

/// List available session names.
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
