//! Session state schema — serialized to `.kairn.state`.

use serde::{Deserialize, Serialize};

/// Current schema version.
pub const SESSION_VERSION: u32 = 3;

/// Persisted workspace state.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionState {
    pub version: u32,
    pub layout: String,
    #[serde(default)]
    pub wide_proportions: Vec<f32>,
    #[serde(default)]
    pub narrow_proportions: Vec<f32>,
    #[serde(default)]
    pub hidden_panels: Vec<usize>,
    pub active_tab: usize,
    pub editor_tabs: Vec<EditorTabState>,
    pub unfolded_dirs: Vec<String>,
    #[serde(default)]
    pub kiro_sessions: Vec<KiroSessionState>,
    #[serde(default)]
    pub split: Option<SplitState>,
}

/// Persisted split panel state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitState {
    /// "horizontal" or "vertical"
    pub direction: String,
    /// Tab indices in the second subpanel (first subpanel has the rest).
    pub second_tabs: Vec<usize>,
    /// Active tab index in first subpanel.
    pub active_first: usize,
    /// Active tab index in second subpanel.
    pub active_second: usize,
    /// Focused subpanel (0 or 1).
    pub focused: usize,
}

/// One editor tab's persisted state.
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorTabState {
    pub path: String,
    pub line: u32,
    pub col: u32,
}

/// Persisted kiro tab session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroSessionState {
    pub name: String,
    pub session_id: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            version: SESSION_VERSION,
            layout: "auto".to_string(),
            wide_proportions: Vec::new(),
            narrow_proportions: Vec::new(),
            hidden_panels: Vec::new(),
            active_tab: 0,
            editor_tabs: Vec::new(),
            unfolded_dirs: Vec::new(),
            kiro_sessions: Vec::new(),
            split: None,
        }
    }
}
