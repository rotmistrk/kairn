//! Undo/redo history for the piece table buffer.
//!
//! Each [`EditRecord`] stores a snapshot of the piece list and line index
//! before an edit. Grouped edits (e.g. `dd`) are collapsed into a single
//! record containing the state before the first edit in the group.

/// A single piece reference — duplicated here to avoid circular deps.
/// Must stay in sync with `piece_table::Piece`.
#[derive(Clone, Debug)]
pub struct PieceSnapshot {
    pub source_is_add: bool,
    pub start: usize,
    pub len: usize,
}

/// A snapshot of buffer state that can be restored on undo.
#[derive(Clone, Debug)]
pub struct EditRecord {
    pub pieces: Vec<PieceSnapshot>,
    pub line_starts: Vec<usize>,
    pub total_len: usize,
}

/// Manages undo and redo stacks with optional grouping.
#[derive(Clone, Debug)]
pub struct UndoHistory {
    undo_stack: Vec<EditRecord>,
    redo_stack: Vec<EditRecord>,
    /// State captured at `begin_group`, applied as a single undo unit.
    group_snapshot: Option<EditRecord>,
}

impl UndoHistory {
    /// Create an empty history.
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            group_snapshot: None,
        }
    }

    /// Push a snapshot before an edit. Clears the redo stack.
    pub fn push(&mut self, record: EditRecord) {
        if self.group_snapshot.is_some() {
            // Inside a group — only the first push matters (already captured).
            return;
        }
        self.undo_stack.push(record);
        self.redo_stack.clear();
    }

    /// Begin a group — the next `push` is captured, subsequent pushes
    /// within the group are ignored.
    pub fn begin_group(&mut self, record: EditRecord) {
        if self.group_snapshot.is_none() {
            self.group_snapshot = Some(record);
            self.redo_stack.clear();
        }
    }

    /// End the current group, committing the captured snapshot.
    pub fn end_group(&mut self) {
        if let Some(snap) = self.group_snapshot.take() {
            self.undo_stack.push(snap);
        }
    }

    /// Pop the last undo record. Returns `None` if nothing to undo.
    pub fn pop_undo(&mut self) -> Option<EditRecord> {
        self.undo_stack.pop()
    }

    /// Push a record onto the redo stack (called during undo).
    pub fn push_redo(&mut self, record: EditRecord) {
        self.redo_stack.push(record);
    }

    /// Pop the last redo record. Returns `None` if nothing to redo.
    pub fn pop_redo(&mut self) -> Option<EditRecord> {
        self.redo_stack.pop()
    }

    /// Push a record onto the undo stack (called during redo).
    pub fn push_undo(&mut self, record: EditRecord) {
        self.undo_stack.push(record);
    }

    /// Number of undoable operations.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Number of redoable operations.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(total_len: usize) -> EditRecord {
        EditRecord {
            pieces: Vec::new(),
            line_starts: vec![0],
            total_len,
        }
    }

    #[test]
    fn push_and_undo() {
        let mut h = UndoHistory::new();
        h.push(make_record(5));
        assert_eq!(h.undo_count(), 1);
        let r = h.pop_undo();
        assert!(r.is_some());
        assert_eq!(r.map(|r| r.total_len), Some(5));
    }

    #[test]
    fn push_clears_redo() {
        let mut h = UndoHistory::new();
        h.push(make_record(5));
        h.push_redo(make_record(10));
        assert_eq!(h.redo_count(), 1);
        h.push(make_record(15));
        assert_eq!(h.redo_count(), 0);
    }

    #[test]
    fn group_collapses() {
        let mut h = UndoHistory::new();
        h.begin_group(make_record(5));
        h.push(make_record(10)); // ignored inside group
        h.push(make_record(15)); // ignored inside group
        h.end_group();
        assert_eq!(h.undo_count(), 1);
        let r = h.pop_undo();
        // Should be the group snapshot (total_len=5), not the pushes
        assert_eq!(r.map(|r| r.total_len), Some(5));
    }

    #[test]
    fn redo_cycle() {
        let mut h = UndoHistory::new();
        h.push(make_record(5));
        let r = h.pop_undo();
        assert!(r.is_some());
        h.push_redo(make_record(10));
        assert_eq!(h.redo_count(), 1);
        let r = h.pop_redo();
        assert!(r.is_some());
    }
}
