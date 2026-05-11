//! Session state schema — serialized to `.kairn.state`.

use serde::{Deserialize, Serialize};

/// Current schema version.
pub const SESSION_VERSION: u32 = 1;

/// Persisted workspace state.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionState {
    pub version: u32,
    pub layout: String,
    pub left_width: u16,
    pub right_width: u16,
    pub active_tab: usize,
    pub editor_tabs: Vec<EditorTabState>,
    pub unfolded_dirs: Vec<String>,
}

/// One editor tab's persisted state.
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorTabState {
    pub path: String,
    pub line: u32,
    pub col: u32,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version: SESSION_VERSION,
            layout: "auto".to_string(),
            left_width: 24,
            right_width: 60,
            active_tab: 0,
            editor_tabs: Vec::new(),
            unfolded_dirs: Vec::new(),
        }
    }
}
