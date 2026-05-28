//! BufferStore trait — abstracts how editor content is persisted.

use std::io;
use std::path::{Path, PathBuf};

pub use crate::command_store::CommandStore;

/// Abstraction for persisting editor buffer content.
pub trait BufferStore: Send {
    /// Persist the current content. Returns error message on failure.
    fn save(&mut self, content: &str) -> Result<(), String>;
}

/// Persists content to a file on disk.
pub struct FileStore {
    path: PathBuf,
}

impl FileStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl BufferStore for FileStore {
    fn save(&mut self, content: &str) -> Result<(), String> {
        write_atomic(&self.path, content).map_err(|e| e.to_string())
    }
}

/// Write content to a file atomically (write to temp, then rename).
fn write_atomic(path: &Path, content: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)
}
