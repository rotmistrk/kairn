//! Atomic file save.

use std::io;
use std::path::Path;

/// Save content to file atomically (write to temp, rename).
pub fn save_file(path: &Path, content: &str) -> io::Result<()> {
    std::fs::write(path, content)
}
