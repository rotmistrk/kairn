//! Shared snapshot of kairn state for MCP tool handlers.

pub use super::cursor_pos::CursorPos;
pub use super::selection_range::SelectionRange;
pub use super::tab_info::TabInfo;
pub use super::terminal_info::TerminalInfo;

/// Snapshot of kairn state, updated on each Tick from the main thread.
#[derive(Debug, Clone, Default)]
pub struct McpSnapshot {
    pub(crate) tabs: Vec<TabInfo>,
    pub(crate) terminals: Vec<TerminalInfo>,
    pub(crate) focused_slot: String,
    pub(crate) messages: Vec<String>,
    /// Content of center-panel tabs (keyed by tab name).
    pub(crate) tab_contents: std::collections::HashMap<String, String>,
    /// Split state: "none", "horizontal", or "vertical".
    pub(crate) split_direction: String,
    pub(crate) split_linked: bool,
}

impl McpSnapshot {
    pub fn tabs(&self) -> &[TabInfo] {
        &self.tabs
    }
    pub fn terminals(&self) -> &[TerminalInfo] {
        &self.terminals
    }
    pub fn focused_slot(&self) -> &str {
        &self.focused_slot
    }
}
