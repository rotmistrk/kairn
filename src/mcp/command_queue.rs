//! Shared command queue + waker for MCP write operations.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use serde_json::Value;
use txv_core::run::Waker;

use super::commands::{McpAction, McpRequest};

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
        let req = McpRequest::new(action, tx);
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
