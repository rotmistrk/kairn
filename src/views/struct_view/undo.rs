//! Snapshot-based undo/redo stack for StructuredView.
//!
//! Stores document states. Position points to the state matching the current doc.
//! save_state() is called BEFORE a mutation with the current (pre-mutation) state.
//! After mutation, the doc diverges from the stack until next save_state or undo.

/// Undo stack storing serialized document snapshots.
pub struct UndoStack {
    /// History of document states.
    snapshots: Vec<String>,
    /// Points past the last saved "before" state. Undo goes back from here.
    position: usize,
    max_size: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
            position: 0,
            max_size: 50,
        }
    }

    /// Save the current state before a mutation. Truncates redo history.
    pub fn save_state(&mut self, state: &str) {
        self.snapshots.truncate(self.position);
        self.snapshots.push(state.to_string());
        self.position = self.snapshots.len();
        // Cap size
        if self.snapshots.len() > self.max_size {
            let excess = self.snapshots.len() - self.max_size;
            self.snapshots.drain(..excess);
            self.position = self.snapshots.len();
        }
    }

    /// Called before the first undo to bookmark the current (post-mutation) state.
    pub fn bookmark_current(&mut self, state: &str) {
        // Only bookmark if we're at the end (no redo available)
        if self.position == self.snapshots.len() {
            self.snapshots.push(state.to_string());
            // Don't advance position — position stays pointing before the bookmark
        }
    }

    /// Move back, return the previous state.
    pub fn undo(&mut self) -> Option<&str> {
        if self.position == 0 {
            return None;
        }
        self.position -= 1;
        Some(&self.snapshots[self.position])
    }

    /// Move forward, return the next state.
    pub fn redo(&mut self) -> Option<&str> {
        if self.position >= self.snapshots.len().saturating_sub(1) {
            return None;
        }
        self.position += 1;
        Some(&self.snapshots[self.position])
    }

    pub fn can_undo(&self) -> bool {
        self.position > 0
    }

    pub fn can_redo(&self) -> bool {
        self.position < self.snapshots.len().saturating_sub(1)
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.position = 0;
    }
}
