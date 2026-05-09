//! FileBroker — tracks open files, prevents duplicates, coordinates focus.

use std::collections::HashMap;

use crate::desktop::SlotId;

/// Result of attempting to open a file.
pub enum OpenResult {
    /// File is already open at this location — just focus it.
    AlreadyOpen { slot: SlotId, tab: usize },
    /// File was newly registered as open.
    Opened,
}

/// Tracks which files are open and where.
pub struct FileBroker {
    open_files: HashMap<String, (SlotId, usize)>,
}

impl FileBroker {
    pub fn new() -> Self {
        Self { open_files: HashMap::new() }
    }

    /// Register a file as open. Returns AlreadyOpen if it was already tracked.
    pub fn open(&mut self, path: &str, slot: SlotId, tab: usize) -> OpenResult {
        if let Some(&(s, t)) = self.open_files.get(path) {
            OpenResult::AlreadyOpen { slot: s, tab: t }
        } else {
            self.open_files.insert(path.to_string(), (slot, tab));
            OpenResult::Opened
        }
    }

    /// Mark a file as closed.
    pub fn close(&mut self, path: &str) {
        self.open_files.remove(path);
    }

    /// Check if a file is currently open.
    pub fn is_open(&self, path: &str) -> bool {
        self.open_files.contains_key(path)
    }

    /// Get all open file paths.
    pub fn open_paths(&self) -> Vec<&str> {
        self.open_files.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for FileBroker {
    fn default() -> Self {
        Self::new()
    }
}
