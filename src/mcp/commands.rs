//! MCP write command queue — allows MCP tools to send mutations to the main thread.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use txv_core::run::Waker;

/// A request from an MCP tool to mutate app state.
pub struct McpRequest {
    pub action: McpAction,
    pub reply: std::sync::mpsc::SyncSender<Result<Value, String>>,
}

/// Actions the MCP server can request.
pub enum McpAction {
    /// Toggle a todo item's completion state. Path is index-based (e.g., [0, 2]).
    TodoToggle { path: Vec<usize> },
    /// Add a todo item as sibling after the given path.
    TodoAdd { path: Vec<usize>, title: String },
    /// Remove a todo item at the given path.
    TodoRemove { path: Vec<usize> },
    /// Move a todo item up within its siblings.
    TodoMoveUp { path: Vec<usize> },
    /// Move a todo item down within its siblings.
    TodoMoveDown { path: Vec<usize> },
    /// Promote a todo item (decrease nesting).
    TodoPromote { path: Vec<usize> },
    /// Demote a todo item (increase nesting).
    TodoDemote { path: Vec<usize> },
    /// Add a subtree of items as children of the item at path.
    TodoAddSubtree {
        path: Vec<usize>,
        items: Vec<serde_json::Value>,
    },
    /// Open an existing file in the editor.
    OpenFile { path: String },
    /// Create a new file on disk and open it.
    CreateFile { path: String, content: String },
    /// Close an editor tab by path/name.
    CloseTab { name: String },
    /// Replace lines in an open buffer.
    EditBuffer {
        name: String,
        start_line: usize,
        end_line: usize,
        text: String,
    },
    /// Insert text at a position in an open buffer.
    InsertText {
        name: String,
        line: usize,
        col: usize,
        text: String,
    },
    /// Move cursor to a position in a tab.
    SetCursor { name: String, line: usize, col: usize },
    /// Save the buffer to disk.
    SaveFile { name: String },
    /// Get diagnostics for a file.
    GetDiagnostics { name: String },
}

/// Shared command queue + waker for MCP write operations.
#[derive(Clone)]
pub struct McpCommandQueue {
    queue: Arc<Mutex<VecDeque<McpRequest>>>,
    waker: Waker,
}

impl McpCommandQueue {
    pub fn new(waker: Waker) -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            waker,
        }
    }

    /// Push a request and wake the event loop. Returns the reply receiver.
    pub fn send(&self, action: McpAction) -> Result<Value, String> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let req = McpRequest { action, reply: tx };
        {
            let mut q = self.queue.lock().map_err(|_| "MCP queue mutex poisoned")?;
            q.push_back(req);
        }
        self.waker.wake();
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|e| format!("MCP command timeout: {e}"))?
    }

    /// Drain pending requests (called from main thread).
    pub fn drain(&self) -> Vec<McpRequest> {
        if let Ok(mut q) = self.queue.lock() {
            q.drain(..).collect()
        } else {
            Vec::new()
        }
    }

    /// Get a handle to the internal queue (for testing).
    pub fn queue_handle(&self) -> &Arc<Mutex<VecDeque<McpRequest>>> {
        &self.queue
    }
}
