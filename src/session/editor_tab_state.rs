//! One editor tab's persisted state.

use serde::{Deserialize, Serialize};

/// One editor tab's persisted state.
#[derive(Debug, Serialize, Deserialize)]
pub struct EditorTabState {
    pub(crate) path: String,
    pub(crate) line: u32,
    pub(crate) col: u32,
}

impl EditorTabState {
    pub fn new(path: impl Into<String>, line: u32, col: u32) -> Self {
        Self {
            path: path.into(),
            line,
            col,
        }
    }
    pub fn path(&self) -> &str {
        &self.path
    }
    pub fn line(&self) -> u32 {
        self.line
    }
    pub fn col(&self) -> u32 {
        self.col
    }
}
