//! Shared snapshot of kairn state for MCP tool handlers.

use serde::Serialize;

/// A tab entry visible to MCP clients.
#[derive(Debug, Clone, Serialize)]
pub struct TabInfo {
    pub name: String,
    pub tab_type: String,
    pub path: Option<String>,
    pub focused: bool,
    pub active: bool,
    pub modified: bool,
    pub cursor: Option<CursorPos>,
    pub selection: Option<SelectionRange>,
    pub order: usize,
}

/// Cursor position in an editor tab.
#[derive(Debug, Clone, Serialize)]
pub struct CursorPos {
    pub line: usize,
    pub col: usize,
}

/// Selection range in an editor tab.
#[derive(Debug, Clone, Serialize)]
pub struct SelectionRange {
    pub start_line: usize,
    pub start_col: usize,
    pub end_line: usize,
    pub end_col: usize,
}

/// A terminal tab entry with content access.
#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub name: String,
    pub terminal_type: String,
    pub index: usize,
    pub content: String,
}

/// Snapshot of kairn state, updated on each Tick from the main thread.
#[derive(Debug, Clone, Default)]
pub struct McpSnapshot {
    pub tabs: Vec<TabInfo>,
    pub terminals: Vec<TerminalInfo>,
    pub focused_slot: String,
    pub messages: Vec<String>,
    /// Content of center-panel tabs (keyed by tab name).
    pub tab_contents: std::collections::HashMap<String, String>,
}
