use serde::Serialize;

/// Cursor position in an editor tab.
#[derive(Debug, Clone, Serialize)]
pub struct CursorPos {
    pub(crate) line: usize,
    pub(crate) col: usize,
}

impl CursorPos {
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn col(&self) -> usize {
        self.col
    }
}
