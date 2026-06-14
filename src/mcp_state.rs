//! MCP-related state extracted from AppState.

use std::sync::mpsc::SyncSender;
use std::sync::{Arc, Mutex};

use serde_json::Value;

use crate::mcp::commands::McpCommandQueue;
use crate::mcp::snapshot::McpSnapshot;

/// MCP snapshot, command queue, and tick counter.
#[derive(Default)]
pub struct McpState {
    /// MCP snapshot (updated periodically for MCP server reads).
    pub(crate) snapshot: Option<Arc<Mutex<McpSnapshot>>>,
    /// MCP command queue for write operations from MCP tools.
    pub(crate) commands: Option<McpCommandQueue>,
    pub(crate) tick: u16,
    /// Pending confirm reply channel (for tool permission prompts).
    pub(crate) pending_confirm_reply: Option<SyncSender<Result<Value, String>>>,
}
