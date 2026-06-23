//! Parameters for async git status collection.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use txv_core::cell::Color;

/// Parameters for async git status collection.
pub(crate) struct GitStatusParams {
    pub(crate) roots: Vec<PathBuf>,
    pub(crate) diff_base: HashMap<PathBuf, String>,
    pub(crate) root_badge_colors: Vec<Color>,
    pub(crate) root_labels: Vec<String>,
    pub(crate) collapsed: HashSet<String>,
}
