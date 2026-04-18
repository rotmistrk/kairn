//! User configuration loaded from ~/.kairn/config.toml

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub default_shell: String,
    pub kiro_command: String,
    pub session_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_shell: "bash".to_string(),
            kiro_command: "kiro-cli".to_string(),
            session_dir: "~/.kairn/sessions".to_string(),
        }
    }
}
