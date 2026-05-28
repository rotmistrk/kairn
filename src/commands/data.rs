//! Data types for command payloads.

use std::path::PathBuf;

/// Data payload for CM_OPEN_FILE / CM_OPEN_FILE_FOCUS commands.
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

/// Payload for CM_CONTENT_CHANGED.
#[derive(Debug, Clone)]
pub struct ContentChanged {
    pub(crate) path: PathBuf,
    pub(crate) content: String,
}

impl ContentChanged {
    pub fn new(path: PathBuf, content: String) -> Self {
        Self { path, content }
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn content(&self) -> &str {
        &self.content
    }
}

/// Context for which confirmation is active.
#[derive(Debug, Clone)]
pub enum ConfirmContext {
    EditorClose(String),
    FileReload(String),
    Quit,
    TodoDelete,
    TodoCrypto,
}

/// Payload for CM_SPLIT.
#[derive(Debug, Clone)]
pub struct SplitRequest {
    pub(crate) vertical: bool,
    pub(crate) file: Option<String>,
}

impl SplitRequest {
    pub fn horizontal() -> Self {
        Self {
            vertical: false,
            file: None,
        }
    }
    pub fn vertical() -> Self {
        Self {
            vertical: true,
            file: None,
        }
    }
    pub fn vertical_with_file(file: String) -> Self {
        Self {
            vertical: true,
            file: Some(file),
        }
    }
    pub fn horizontal_with_file(file: String) -> Self {
        Self {
            vertical: false,
            file: Some(file),
        }
    }
    pub fn is_vertical(&self) -> bool {
        self.vertical
    }
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }
}

/// Payload for CM_DIFF_SPLIT — side-by-side diff.
#[derive(Debug, Clone)]
pub struct DiffSplitRequest {
    pub(crate) base_content: String,
    pub(crate) base_ref: String,
}

impl DiffSplitRequest {
    pub fn new(base_content: impl Into<String>, base_ref: impl Into<String>) -> Self {
        Self {
            base_content: base_content.into(),
            base_ref: base_ref.into(),
        }
    }
    pub fn base_content(&self) -> &str {
        &self.base_content
    }
    pub fn base_ref(&self) -> &str {
        &self.base_ref
    }
}
