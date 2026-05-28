//! PluginEntry — state of a loaded plugin.

use std::path::PathBuf;
use std::time::SystemTime;

/// State of a loaded plugin.
#[derive(Debug)]
pub(super) struct PluginEntry {
    pub(super) path: PathBuf,
    pub(super) mtime: SystemTime,
    /// Proc names defined by this plugin.
    pub(super) procs: Vec<String>,
}
