//! BufferRegistry — tracks buffer IDs and metadata for split-view support.
//!
//! The registry assigns unique IDs to buffers and tracks their file paths
//! and reference counts. This enables multiple views of the same buffer
//! (split views) by providing a shared identity for each buffer.
//!
//! In this first step, the actual PieceTable remains owned by Editor.
//! The registry is the source of truth for buffer lifecycle and identity.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::buffer_entry::BufferEntry;
pub use crate::buffer_id::BufferId;

/// Registry that assigns IDs and tracks buffer metadata.
pub struct BufferRegistry {
    entries: HashMap<BufferId, BufferEntry>,
    path_index: HashMap<PathBuf, BufferId>,
    next_id: u64,
}

impl BufferRegistry {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            path_index: HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new buffer with an optional file path. Returns its unique ID.
    pub fn register(&mut self, path: Option<PathBuf>) -> BufferId {
        let id = BufferId::new(self.next_id);
        self.next_id += 1;
        if let Some(ref p) = path {
            self.path_index.insert(p.clone(), id);
        }
        self.entries.insert(id, BufferEntry { path, ref_count: 1 });
        id
    }

    /// Look up a buffer ID by file path.
    pub fn find_by_path(&self, path: &PathBuf) -> Option<BufferId> {
        self.path_index.get(path).copied()
    }

    /// Get the file path for a buffer.
    pub fn path(&self, id: BufferId) -> Option<&PathBuf> {
        self.entries.get(&id).and_then(|e| e.path.as_ref())
    }

    /// Increment reference count (another view opened this buffer).
    pub fn add_ref(&mut self, id: BufferId) {
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.ref_count += 1;
        }
    }

    /// Decrement reference count. Returns true if buffer was removed (ref_count hit 0).
    pub fn release(&mut self, id: BufferId) -> bool {
        let remove = if let Some(entry) = self.entries.get_mut(&id) {
            entry.ref_count = entry.ref_count.saturating_sub(1);
            entry.ref_count == 0
        } else {
            return false;
        };
        if remove {
            if let Some(entry) = self.entries.remove(&id) {
                if let Some(ref p) = entry.path {
                    self.path_index.remove(p);
                }
            }
            true
        } else {
            false
        }
    }

    /// Get the reference count for a buffer.
    pub fn ref_count(&self, id: BufferId) -> usize {
        self.entries.get(&id).map_or(0, |e| e.ref_count)
    }

    /// Check if a buffer ID is registered.
    pub fn contains(&self, id: BufferId) -> bool {
        self.entries.contains_key(&id)
    }

    /// Update the path for a buffer (e.g., after "save as").
    pub fn set_path(&mut self, id: BufferId, path: PathBuf) {
        // Remove old path index entry
        if let Some(entry) = self.entries.get(&id) {
            if let Some(ref old_path) = entry.path {
                self.path_index.remove(old_path);
            }
        }
        self.path_index.insert(path.clone(), id);
        if let Some(entry) = self.entries.get_mut(&id) {
            entry.path = Some(path);
        }
    }
}

impl Default for BufferRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_assigns_unique_ids() {
        let mut reg = BufferRegistry::new();
        let id1 = reg.register(None);
        let id2 = reg.register(None);
        assert_ne!(id1, id2);
    }

    #[test]
    fn find_by_path_returns_registered_id() {
        let mut reg = BufferRegistry::new();
        let path = PathBuf::from("/tmp/test.rs");
        let id = reg.register(Some(path.clone()));
        assert_eq!(reg.find_by_path(&path), Some(id));
    }

    #[test]
    fn release_removes_at_zero_refs() {
        let mut reg = BufferRegistry::new();
        let path = PathBuf::from("/tmp/test.rs");
        let id = reg.register(Some(path.clone()));
        assert!(reg.release(id));
        assert!(!reg.contains(id));
        assert_eq!(reg.find_by_path(&path), None);
    }

    #[test]
    fn add_ref_prevents_removal() {
        let mut reg = BufferRegistry::new();
        let id = reg.register(None);
        reg.add_ref(id);
        assert_eq!(reg.ref_count(id), 2);
        assert!(!reg.release(id));
        assert!(reg.contains(id));
        assert!(reg.release(id));
        assert!(!reg.contains(id));
    }

    #[test]
    fn set_path_updates_index() {
        let mut reg = BufferRegistry::new();
        let old = PathBuf::from("/tmp/old.rs");
        let new = PathBuf::from("/tmp/new.rs");
        let id = reg.register(Some(old.clone()));
        reg.set_path(id, new.clone());
        assert_eq!(reg.find_by_path(&old), None);
        assert_eq!(reg.find_by_path(&new), Some(id));
        assert_eq!(reg.path(id), Some(&new));
    }
}
