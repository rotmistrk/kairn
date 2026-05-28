//! Shared snapshot of kairn state for MCP tool handlers.

use serde::Serialize;

/// A tab entry visible to MCP clients.
#[derive(Debug, Clone, Serialize)]
pub struct TabInfo {
    pub(crate) name: String,
    pub(crate) tab_type: String,
    pub(crate) path: Option<String>,
    pub(crate) focused: bool,
    pub(crate) active: bool,
    pub(crate) modified: bool,
    pub(crate) cursor: Option<CursorPos>,
    pub(crate) selection: Option<SelectionRange>,
    pub(crate) order: usize,
}

impl TabInfo {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn tab_type(&self) -> &str {
        &self.tab_type
    }
    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
    pub fn focused(&self) -> bool {
        self.focused
    }
    pub fn modified(&self) -> bool {
        self.modified
    }
    pub fn cursor(&self) -> Option<&CursorPos> {
        self.cursor.as_ref()
    }
}

/// Cursor position in an editor tab.
#[derive(Debug, Clone, Serialize)]
pub struct CursorPos {
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl CursorPos {
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn col(&self) -> usize {
        self.col
    }
}

/// Selection range in an editor tab.
#[derive(Debug, Clone, Serialize)]
pub struct SelectionRange {
    pub(crate) start_line: usize,
    pub(crate) start_col: usize,
    pub(crate) end_line: usize,
    pub(crate) end_col: usize,
}

/// A terminal tab entry with content access.
#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub(crate) name: String,
    pub(crate) terminal_type: String,
    pub(crate) index: usize,
    pub(crate) content: String,
}

impl TerminalInfo {
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn terminal_type(&self) -> &str {
        &self.terminal_type
    }
    pub fn index(&self) -> usize {
        self.index
    }
    pub fn content(&self) -> &str {
        &self.content
    }
}

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
