//! Shared snapshot of kairn state for MCP tool handlers.

use serde::Serialize;

/// A tab entry visible to MCP clients.
#[derive(Debug, Clone, Serialize)]
pub struct TabInfo {
    pub name: String,
    pub tab_type: String, // "shell", "kiro", "editor"
    pub path: Option<String>,
}

/// A terminal tab entry with content access.
#[derive(Debug, Clone, Serialize)]
pub struct TerminalInfo {
    pub name: String,
    pub terminal_type: String, // "shell" or "kiro"
    pub content: String,
}

/// Snapshot of kairn state, updated on each Tick from the main thread.
#[derive(Debug, Clone, Default)]
pub struct McpSnapshot {
    pub tabs: Vec<TabInfo>,
    pub terminals: Vec<TerminalInfo>,
}
