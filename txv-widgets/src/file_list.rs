//! FileListData — ListData implementation for flat file listing.
//! Uses the `ignore` crate to respect .gitignore rules.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use txv_core::prelude::*;

use crate::list_view::ListData;

struct FileEntry {
    path: PathBuf,
    display: String,
}

/// Flat file list data provider (recursive walk).
pub struct FileListData {
    root: PathBuf,
    entries: Vec<FileEntry>,
}

impl FileListData {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let mut entries = Vec::new();
        let walker = WalkBuilder::new(&root).sort_by_file_name(|a, b| a.cmp(b)).build();

        for entry in walker.flatten() {
            let path = entry.path().to_path_buf();
            if path == root || path.is_dir() {
                continue;
            }
            let display = path.strip_prefix(&root).unwrap_or(&path).to_string_lossy().into_owned();
            entries.push(FileEntry { path, display });
        }

        Self { root, entries }
    }

    pub fn path(&self, index: usize) -> &Path {
        &self.entries[index].path
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn refresh(&mut self) {
        *self = Self::new(self.root.clone());
    }
}

impl ListData for FileListData {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn label(&self, index: usize) -> &str {
        &self.entries[index].display
    }

    fn style(&self, _index: usize) -> Style {
        Style::default()
    }
}
