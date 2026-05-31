//! Session state schema — serialized to `.kairn.state`.

use serde::{Deserialize, Serialize};

pub use super::editor_tab_state::EditorTabState;
pub use super::kiro_session_state::KiroSessionState;
pub use super::session_state_builder::SessionStateBuilder;
pub use super::split_state::SplitState;

/// Current schema version.
pub const SESSION_VERSION: u32 = 4;

/// Persisted workspace state.
#[derive(Debug, Serialize, Deserialize)]
pub struct SessionState {
    pub(crate) version: u32,
    pub(crate) layout: String,
    #[serde(default)]
    pub(crate) wide_proportions: Vec<f32>,
    #[serde(default)]
    pub(crate) narrow_proportions: Vec<f32>,
    #[serde(default)]
    pub(crate) hidden_panels: Vec<usize>,
    pub(crate) active_tab: usize,
    pub(crate) editor_tabs: Vec<EditorTabState>,
    pub(crate) unfolded_dirs: Vec<String>,
    #[serde(default)]
    pub(crate) kiro_sessions: Vec<KiroSessionState>,
    #[serde(default)]
    pub(crate) split: Option<SplitState>,
    /// Additional workspace root directories (absolute paths).
    #[serde(default)]
    pub(crate) roots: Vec<String>,
}

impl SessionState {
    pub fn editor_tabs(&self) -> &[EditorTabState] {
        &self.editor_tabs
    }
    pub fn kiro_sessions(&self) -> &[KiroSessionState] {
        &self.kiro_sessions
    }
    pub fn layout(&self) -> &str {
        &self.layout
    }
    pub fn version(&self) -> u32 {
        self.version
    }
    pub fn wide_proportions(&self) -> &[f32] {
        &self.wide_proportions
    }
    pub fn narrow_proportions(&self) -> &[f32] {
        &self.narrow_proportions
    }
    pub fn hidden_panels(&self) -> &[usize] {
        &self.hidden_panels
    }
    pub fn active_tab(&self) -> usize {
        self.active_tab
    }
    pub fn unfolded_dirs(&self) -> &[String] {
        &self.unfolded_dirs
    }
    pub fn split(&self) -> Option<&SplitState> {
        self.split.as_ref()
    }

    pub fn roots(&self) -> &[String] {
        &self.roots
    }

    pub fn builder() -> SessionStateBuilder {
        SessionStateBuilder::default()
    }
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
            roots: Vec::new(),
        }
    }
}
