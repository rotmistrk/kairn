//! WorkspaceRoot — a single root directory with its badge color.

use std::path::{Path, PathBuf};

use txv_core::cell::Color;

/// A workspace root directory with its assigned badge color.
#[derive(Clone, Debug)]
pub struct WorkspaceRoot {
    pub(crate) path: PathBuf,
    pub(crate) color: Color,
}

impl WorkspaceRoot {
    pub(crate) fn new(path: PathBuf, color: Color) -> Self {
        Self { path, color }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn color(&self) -> Color {
        self.color
    }
}
