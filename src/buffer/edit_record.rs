//! EditRecord — a snapshot of buffer state for undo.

use super::piece_table::Piece;

/// A snapshot of buffer state for undo.
#[derive(Clone)]
pub struct EditRecord {
    pub(super) pieces: Vec<Piece>,
    pub(crate) line_starts: Vec<usize>,
}
