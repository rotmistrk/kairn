//! BufferEntry — metadata for a registered buffer.

use std::path::PathBuf;

/// Metadata for a registered buffer.
pub(crate) struct BufferEntry {
    pub(crate) path: Option<PathBuf>,
    pub(crate) ref_count: usize,
}
