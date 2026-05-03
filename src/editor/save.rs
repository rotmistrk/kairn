/// Atomic file save — writes to a temp file then renames.
///
/// This prevents data loss if the process is interrupted mid-write.
use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result};

/// Atomically save `content` to `path`.
///
/// Writes to a temporary file in the same directory, then renames it
/// over the target. This ensures the file is never partially written.
pub fn atomic_save(path: &str, content: &str) -> Result<()> {
    let target = Path::new(path);
    let dir = target.parent().unwrap_or_else(|| Path::new("."));
    let tmp_path = dir.join(format!(".kairn-save-{}.tmp", std::process::id()));
    // Write to temp file
    let mut f = fs::File::create(&tmp_path)
        .with_context(|| format!("create temp file: {}", tmp_path.display()))?;
    f.write_all(content.as_bytes())
        .with_context(|| format!("write to temp file: {}", tmp_path.display()))?;
    f.sync_all().with_context(|| "sync temp file")?;
    drop(f);
    // Rename over target (atomic on same filesystem)
    fs::rename(&tmp_path, target)
        .with_context(|| format!("rename {} → {}", tmp_path.display(), target.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn save_and_read_back() {
        let dir = std::env::temp_dir().join("kairn-test-save");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.txt");
        let path_str = path.to_str().unwrap_or("");
        atomic_save(path_str, "hello world\n").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "hello world\n");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn overwrite_existing() {
        let dir = std::env::temp_dir().join("kairn-test-save2");
        let _ = fs::create_dir_all(&dir);
        let path = dir.join("test.txt");
        let path_str = path.to_str().unwrap_or("");
        fs::write(&path, "old content").unwrap();
        atomic_save(path_str, "new content").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert_eq!(content, "new content");
        let _ = fs::remove_dir_all(&dir);
    }
}
