//! A single result entry (file + location + context text).

use std::path::PathBuf;

/// A single result entry (file + location + context text).
#[derive(Debug, Clone)]
pub struct ResultEntry {
    pub(crate) path: PathBuf,
    pub(crate) line: u32,
    pub(crate) col: u32,
    pub(crate) text: String,
}

impl ResultEntry {
    pub fn new(path: PathBuf, line: u32, col: u32, text: impl Into<String>) -> Self {
        Self {
            path,
            line,
            col,
            text: text.into(),
        }
    }
}
