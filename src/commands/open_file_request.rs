//! Data payload for CM_OPEN_FILE / CM_OPEN_FILE_FOCUS commands.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct OpenFileRequest {
    pub(crate) path: PathBuf,
    pub(crate) line: Option<u32>,
    pub(crate) col: Option<u32>,
    pub(crate) diff: bool,
    /// Custom diff base ref (e.g. a commit hash). Used when opening from git pane with a base set.
    pub(crate) diff_base: Option<String>,
}

impl OpenFileRequest {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            line: None,
            col: None,
            diff: false,
            diff_base: None,
        }
    }
    pub fn at(path: PathBuf, line: u32, col: u32) -> Self {
        Self {
            path,
            line: Some(line),
            col: Some(col),
            diff: false,
            diff_base: None,
        }
    }
    pub fn with_diff(path: PathBuf) -> Self {
        Self {
            path,
            line: None,
            col: None,
            diff: true,
            diff_base: None,
        }
    }
    pub fn with_diff_base(path: PathBuf, base: String) -> Self {
        Self {
            path,
            line: None,
            col: None,
            diff: true,
            diff_base: Some(base),
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
    pub fn diff_base(&self) -> Option<&str> {
        self.diff_base.as_deref()
    }
}
