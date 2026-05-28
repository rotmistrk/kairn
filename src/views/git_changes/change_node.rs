//! A node in the git changes tree.

use std::path::PathBuf;

use txv_core::cell::Color;

use crate::git_status::FileStatus;

/// A node in the git changes tree.
#[derive(Clone)]
pub(super) struct ChangeNode {
    pub(super) label: String,
    pub(super) depth: usize,
    pub(super) expandable: bool,
    pub(super) expanded: bool,
    pub(super) file_path: Option<PathBuf>,
    pub(super) color: Color,
    pub(super) status: Option<FileStatus>,
}
