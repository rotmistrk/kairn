//! Buffer module — PieceTable text buffer with undo.

pub mod edit_record;
pub mod line_index;
pub mod piece_table;
pub mod undo;

pub use edit_record::EditRecord;
pub use piece_table::PieceTable;
