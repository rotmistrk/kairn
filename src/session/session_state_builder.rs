//! Builder for SessionState.

use super::editor_tab_state::EditorTabState;
use super::kiro_session_state::KiroSessionState;
use super::schema::SessionState;
use super::split_state::SplitState;

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
