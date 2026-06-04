//! Ephemeral highlights — transient visual markers for goto, search, multicursor.

use super::ephemeral_range::EphemeralRange;

/// Who owns these highlights (determines clear policy).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HighlightOwner {
    /// Cleared automatically on any cursor movement or editing keystroke.
    Transient,
    /// Cleared only when search mode exits.
    Search,
    /// Cleared only on explicit Esc (future multicursor).
    Multicursor,
}

/// The set of ephemeral highlights on an editor.
pub struct EphemeralHighlights {
    pub(crate) ranges: Vec<EphemeralRange>,
    pub(crate) owner: HighlightOwner,
}

impl Default for EphemeralHighlights {
    fn default() -> Self {
        Self::new()
    }
}

impl EphemeralHighlights {
    pub fn new() -> Self {
        Self {
            ranges: Vec::new(),
            owner: HighlightOwner::Transient,
        }
    }

    /// Set highlights with the given owner, replacing any existing.
    pub fn set(&mut self, ranges: Vec<EphemeralRange>, owner: HighlightOwner) {
        self.ranges = ranges;
        self.owner = owner;
    }

    /// Clear transient highlights. Returns true if anything was cleared.
    pub fn clear_transient(&mut self) -> bool {
        if self.owner == HighlightOwner::Transient && !self.ranges.is_empty() {
            self.ranges.clear();
            return true;
        }
        false
    }

    /// Clear all highlights unconditionally.
    pub fn clear_all(&mut self) {
        self.ranges.clear();
        self.owner = HighlightOwner::Transient;
    }

    /// Are there any active highlights?
    pub fn is_empty(&self) -> bool {
        self.ranges.is_empty()
    }
}
