//! ClipEntry — a single clipboard ring entry.

use std::time::Instant;

/// A single clipboard entry.
#[derive(Clone)]
pub struct ClipEntry {
    pub(crate) text: String,
    pub(crate) source: String,
    pub(crate) timestamp: Instant,
    pub(crate) line_count: usize,
}
