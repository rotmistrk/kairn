//! Buffer module — PieceTable text buffer with undo.

pub mod line_index;
pub mod piece_table;
pub mod undo;

pub use piece_table::PieceTable;
