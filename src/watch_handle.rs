//! Per-consumer handle that tracks last-seen generation from GitWatcher.

use std::sync::Arc;

use crate::git_watcher::GitWatcher;

/// Per-consumer handle that tracks last-seen generation.
pub struct WatchHandle {
    watcher: Arc<GitWatcher>,
    last_gen: u64,
}

impl WatchHandle {
    pub fn new(watcher: Arc<GitWatcher>) -> Self {
        let last_gen = watcher.generation();
        Self { watcher, last_gen }
    }

    /// Returns true if changes occurred since last check. Updates internal state.
    pub fn has_changes(&mut self) -> bool {
        let current = self.watcher.generation();
        if current != self.last_gen {
            self.last_gen = current;
            return true;
        }
        false
    }

    /// Signal a change on the underlying watcher.
    pub fn signal_change(&self) {
        self.watcher.signal_change();
    }
}
