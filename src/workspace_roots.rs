//! Workspace root management — multi-root directory support.

use std::path::{Path, PathBuf};

use crate::app_palette::app_palette;
use crate::workspace_root::WorkspaceRoot;

/// Collection of workspace roots with color assignment and lookup.
#[derive(Clone, Debug)]
pub struct WorkspaceRoots {
    roots: Vec<WorkspaceRoot>,
}

impl WorkspaceRoots {
    /// Create with a single primary root.
    pub fn new(primary: PathBuf) -> Self {
        let color = app_palette().roots().color_at(0);
        Self {
            roots: vec![WorkspaceRoot::new(primary, color)],
        }
    }

    /// All roots (sorted alphabetically).
    pub fn all(&self) -> &[WorkspaceRoot] {
        &self.roots
    }

    /// Find which root a file path belongs to (longest prefix match).
    pub fn root_for(&self, path: &Path) -> &WorkspaceRoot {
        self.roots
            .iter()
            .filter(|r| path.starts_with(r.path()))
            .max_by_key(|r| r.path().as_os_str().len())
            .unwrap_or(&self.roots[0])
    }

    /// Add a new root. Returns false if already present.
    pub fn add(&mut self, path: PathBuf) -> bool {
        if self.roots.iter().any(|r| r.path() == path) {
            return false;
        }
        let color = app_palette().roots().color_at(self.roots.len());
        self.roots.push(WorkspaceRoot::new(path, color));
        self.roots.sort_by(|a, b| a.path().cmp(b.path()));
        true
    }

    /// Remove a root. Returns false if not found or if it's the last root.
    pub fn remove(&mut self, path: &Path) -> bool {
        if self.roots.len() <= 1 {
            return false;
        }
        let before = self.roots.len();
        self.roots.retain(|r| r.path() != path);
        self.roots.len() < before
    }

    /// Number of roots.
    pub fn len(&self) -> usize {
        self.roots.len()
    }

    /// Whether there are no roots (should never be true in practice).
    pub fn is_empty(&self) -> bool {
        self.roots.is_empty()
    }

    /// Root paths for serialization.
    pub fn paths(&self) -> Vec<&Path> {
        self.roots.iter().map(|r| r.path()).collect()
    }

    /// Restore from saved paths.
    pub fn restore(&mut self, paths: Vec<PathBuf>) {
        let palette = app_palette();
        self.roots.clear();
        for (i, path) in paths.into_iter().enumerate() {
            let color = palette.roots().color_at(i);
            self.roots.push(WorkspaceRoot::new(path, color));
        }
        self.roots.sort_by(|a, b| a.path().cmp(b.path()));
    }
}
