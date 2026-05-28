//! UndoHistory — snapshot-based undo/redo.

use super::piece_table::Piece;

/// A snapshot of buffer state for undo.
#[derive(Clone)]
pub struct EditRecord {
    pub(super) pieces: Vec<Piece>,
    pub(crate) line_starts: Vec<usize>,
}

/// Undo/redo stack.
pub struct UndoHistory {
    undo_stack: Vec<EditRecord>,
    redo_stack: Vec<EditRecord>,
}

impl Default for UndoHistory {
    fn default() -> Self {
        Self::new()
    }
}

impl UndoHistory {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn push(&mut self, record: EditRecord) {
        self.undo_stack.push(record);
        self.redo_stack.clear();
    }

    pub fn undo(&mut self, current: EditRecord) -> Option<EditRecord> {
        let prev = self.undo_stack.pop()?;
        self.redo_stack.push(current);
        Some(prev)
    }

    pub fn redo(&mut self, current: EditRecord) -> Option<EditRecord> {
        let next = self.redo_stack.pop()?;
        self.undo_stack.push(current);
        Some(next)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
