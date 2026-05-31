//! Data payload for CM_ROOTS_CHANGED broadcast.

use std::path::{PathBuf, MAIN_SEPARATOR};

use txv_core::cell::Color;
use txv_core::disambiguate::{disambiguate, Side};

use crate::workspace_roots::WorkspaceRoots;

/// Payload for CM_ROOTS_CHANGED: paths + badge colors + disambiguated labels.
pub struct RootsChangedData {
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) colors: Vec<Color>,
    /// Disambiguated display labels for each root (shortest unique directory name).
    pub(crate) labels: Vec<String>,
}

impl RootsChangedData {
    pub fn new(paths: Vec<PathBuf>, colors: Vec<Color>, labels: Vec<String>) -> Self {
        Self { paths, colors, labels }
    }
    pub fn from_roots(roots: &WorkspaceRoots) -> Self {
        let paths: Vec<PathBuf> = roots.paths().iter().map(|p| p.to_path_buf()).collect();
        let path_strs: Vec<String> = paths.iter().map(|p| p.to_string_lossy().to_string()).collect();
        let refs: Vec<&str> = path_strs.iter().map(|s| s.as_str()).collect();
        let labels = disambiguate(&refs, MAIN_SEPARATOR, Side::Right);
        Self {
            paths,
            colors: roots.all().iter().map(|r| r.color()).collect(),
            labels,
        }
    }
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }
    pub fn labels(&self) -> &[String] {
        &self.labels
    }
}
