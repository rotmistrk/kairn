//! TaskOutput — shared async output state for background tasks (grep, build).

use std::sync::{Arc, Mutex};

use crate::views::results::ResultEntry;

/// Shared state between a background task thread and the UI handler.
pub struct TaskOutput {
    entries: Mutex<Vec<ResultEntry>>,
    done: Mutex<bool>,
    error: Mutex<Option<String>>,
    exit_code: Mutex<Option<i32>>,
}

impl TaskOutput {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            entries: Mutex::new(Vec::new()),
            done: Mutex::new(false),
            error: Mutex::new(None),
            exit_code: Mutex::new(None),
        })
    }

    pub fn push_entries(&self, batch: &mut Vec<ResultEntry>) {
        if let Ok(mut v) = self.entries.lock() {
            v.append(batch);
        }
    }

    pub fn take_entries(&self) -> Vec<ResultEntry> {
        self.entries
            .lock()
            .map(|mut v| std::mem::take(&mut *v))
            .unwrap_or_default()
    }

    pub fn mark_done(&self) {
        if let Ok(mut d) = self.done.lock() {
            *d = true;
        }
    }

    pub fn is_done(&self) -> bool {
        self.done.lock().map(|d| *d).unwrap_or(false)
    }

    pub fn set_error(&self, msg: String) {
        if let Ok(mut e) = self.error.lock() {
            *e = Some(msg);
        }
    }

    pub fn take_error(&self) -> Option<String> {
        self.error.lock().ok().and_then(|mut e| e.take())
    }

    pub fn set_exit_code(&self, code: i32) {
        if let Ok(mut c) = self.exit_code.lock() {
            *c = Some(code);
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code.lock().ok().and_then(|c| *c)
    }
}
