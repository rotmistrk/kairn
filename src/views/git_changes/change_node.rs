//! A node in the git changes tree.

use std::path::PathBuf;

use txv_core::cell::Color;

use crate::git_status::FileStatus;

/// A node in the git changes tree.
#[derive(Clone)]
pub(crate) struct ChangeNode {
    pub(crate) label: String,
    pub(crate) depth: usize,
    pub(crate) expandable: bool,
    pub(crate) expanded: bool,
    pub(crate) file_path: Option<PathBuf>,
    pub(crate) color: Color,
    pub(crate) status: Option<FileStatus>,
    /// Stable identity key for preserving expand state across rebuilds.
    /// Root headers: absolute path. Categories: "{root_path}:{status_name}".
    pub(crate) key: Option<String>,
}
