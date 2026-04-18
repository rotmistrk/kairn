//! Configuration loaded from .kairnrc search path.
//! Search order: $PWD/.kairnrc → $HOME/.kairnrc → defaults.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_shell")]
    pub default_shell: String,
    #[serde(default = "default_kiro_command")]
    pub kiro_command: String,
}

fn default_shell() -> String {
    "bash".to_string()
}
fn default_kiro_command() -> String {
    "kiro-cli".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_shell: default_shell(),
            kiro_command: default_kiro_command(),
        }
    }
}

impl Config {
    /// Load config from .kairnrc search path.
    /// $PWD/.kairnrc overrides $HOME/.kairnrc.
    pub fn load(workspace: &Path) -> Self {
        let candidates = rc_search_path(workspace);
        for path in &candidates {
            if let Ok(content) = std::fs::read_to_string(path) {
                if let Ok(cfg) = serde_json::from_str(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }
}

fn rc_search_path(workspace: &Path) -> Vec<PathBuf> {
    let mut paths = vec![workspace.join(".kairnrc")];
    if let Ok(home) = std::env::var("HOME") {
        paths.push(PathBuf::from(home).join(".kairnrc"));
    }
    paths
}
