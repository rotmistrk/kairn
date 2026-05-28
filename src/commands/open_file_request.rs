//! Data payload for CM_OPEN_FILE / CM_OPEN_FILE_FOCUS commands.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct OpenFileRequest {
    pub(crate) path: PathBuf,
    pub(crate) line: Option<u32>,
    pub(crate) col: Option<u32>,
    pub(crate) diff: bool,
}

impl OpenFileRequest {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            line: None,
            col: None,
            diff: false,
        }
    }
    pub fn at(path: PathBuf, line: u32, col: u32) -> Self {
        Self {
            path,
            line: Some(line),
            col: Some(col),
            diff: false,
        }
    }
    pub fn with_diff(path: PathBuf) -> Self {
        Self {
            path,
            line: None,
            col: None,
            diff: true,
        }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn line(&self) -> Option<u32> {
        self.line
    }
    pub fn col(&self) -> Option<u32> {
        self.col
    }
    pub fn diff(&self) -> bool {
        self.diff
    }
}
