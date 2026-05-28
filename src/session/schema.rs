//! Session state schema — serialized to `.kairn.state`.

use serde::{Deserialize, Serialize};

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
}

/// Persisted split panel state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SplitState {
    /// "horizontal" or "vertical"
    pub(crate) direction: String,
    /// Tab indices in the second subpanel (first subpanel has the rest).
    pub(crate) second_tabs: Vec<usize>,
    /// Active tab index in first subpanel.
    pub(crate) active_first: usize,
    /// Active tab index in second subpanel.
    pub(crate) active_second: usize,
    /// Focused subpanel (0 or 1).
    pub(crate) focused: usize,
}

/// One editor tab's persisted state.
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorTabState {
    pub(crate) path: String,
    pub(crate) line: u32,
    pub(crate) col: u32,
}

/// Persisted kiro tab session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KiroSessionState {
    pub(crate) name: String,
    pub(crate) session_id: Option<String>,
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

    pub fn builder() -> SessionStateBuilder {
        SessionStateBuilder::default()
    }
}

#[derive(Default)]
pub struct SessionStateBuilder {
    state: SessionState,
}

impl SessionStateBuilder {
    pub fn version(mut self, v: u32) -> Self {
        self.state.version = v;
        self
    }
    pub fn layout(mut self, l: impl Into<String>) -> Self {
        self.state.layout = l.into();
        self
    }
    pub fn active_tab(mut self, t: usize) -> Self {
        self.state.active_tab = t;
        self
    }
    pub fn editor_tabs(mut self, tabs: Vec<EditorTabState>) -> Self {
        self.state.editor_tabs = tabs;
        self
    }
    pub fn unfolded_dirs(mut self, dirs: Vec<String>) -> Self {
        self.state.unfolded_dirs = dirs;
        self
    }
    pub fn kiro_sessions(mut self, s: Vec<KiroSessionState>) -> Self {
        self.state.kiro_sessions = s;
        self
    }
    pub fn split(mut self, s: SplitState) -> Self {
        self.state.split = Some(s);
        self
    }
    pub fn build(self) -> SessionState {
        self.state
    }
}

impl EditorTabState {
    pub fn new(path: impl Into<String>, line: u32, col: u32) -> Self {
        Self {
            path: path.into(),
            line,
            col,
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn col(&self) -> u32 {
        self.col
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
        }
    }
}
