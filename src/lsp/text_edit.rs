//! LSP TextEdit — a single text replacement from the server.

/// A single text edit from additional edits (e.g. auto-import).
#[derive(Debug, Clone)]
pub struct TextEdit {
    pub(crate) start_line: u32,
    pub(crate) start_col: u32,
    pub(crate) end_line: u32,
    pub(crate) end_col: u32,
    pub(crate) new_text: String,
}
