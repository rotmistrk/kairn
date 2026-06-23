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
    snapshot: Option<Arc<Mutex<McpSnapshot>>>,
    /// MCP command queue for write operations from MCP tools.
    commands: Option<McpCommandQueue>,
    tick: u16,
    /// Pending confirm reply channel (for tool permission prompts).
    pending_confirm_reply: Option<SyncSender<Result<Value, String>>>,
}

impl McpState {
    pub(crate) fn snapshot(&self) -> &Option<Arc<Mutex<McpSnapshot>>> {
        &self.snapshot
    }

    pub(crate) fn set_snapshot(&mut self, s: Arc<Mutex<McpSnapshot>>) {
        self.snapshot = Some(s);
    }

    pub(crate) fn commands(&self) -> &Option<McpCommandQueue> {
        &self.commands
    }

    pub(crate) fn set_commands(&mut self, q: McpCommandQueue) {
        self.commands = Some(q);
    }

    pub(crate) fn tick(&self) -> u16 {
        self.tick
    }

    pub(crate) fn increment_tick(&mut self) {
        self.tick = self.tick.wrapping_add(1);
    }

    pub(crate) fn take_confirm_reply(&mut self) -> Option<SyncSender<Result<Value, String>>> {
        self.pending_confirm_reply.take()
    }

    pub(crate) fn set_confirm_reply(&mut self, reply: SyncSender<Result<Value, String>>) {
        self.pending_confirm_reply = Some(reply);
    }
}
