//! Payload for CM_CONTENT_CHANGED.

use std::path::PathBuf;

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
