//! GitWatcher — reactive file watching for git status changes.
//! Uses notify crate (inotify/kqueue) to detect changes without polling.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

/// Watches git-relevant paths and signals when status may have changed.
pub struct GitWatcher {
    changed: Arc<AtomicBool>,
    _watcher: RecommendedWatcher,
}

impl GitWatcher {
    /// Create a watcher for the given project root.
    /// Watches .git/index, .git/refs, and the working tree.
    /// Returns None if watcher creation fails.
    pub fn new(root: &Path) -> Option<Self> {
        let git_dir = find_git_dir(root)?;
        let changed = Arc::new(AtomicBool::new(false));
        let flag = changed.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if is_relevant(&event) {
                    flag.store(true, Ordering::Relaxed);
                }
            }
        })
        .ok()?;

        // Watch .git/index and .git/refs for commit/stage changes
        let _ = watcher.watch(&git_dir.join("index"), RecursiveMode::NonRecursive);
        let _ = watcher.watch(&git_dir.join("refs"), RecursiveMode::Recursive);

        // Watch working tree for file modifications (non-recursive at root,
        // rely on notify's recursive mode for subdirs)
        let _ = watcher.watch(root, RecursiveMode::Recursive);

        Some(Self {
            changed,
            _watcher: watcher,
        })
    }

    /// Non-blocking check: returns true if changes detected since last call.
    /// Resets the flag on read.
    pub fn has_changes(&self) -> bool {
        self.changed.swap(false, Ordering::Relaxed)
    }

    /// Force a change signal (e.g., on CM_SAVE).
    pub fn signal_change(&self) {
        self.changed.store(true, Ordering::Relaxed);
    }
}

/// Find the .git directory for a given root.
fn find_git_dir(root: &Path) -> Option<PathBuf> {
    let git_path = root.join(".git");
    if git_path.is_dir() {
        return Some(git_path);
    }
    // Handle git worktrees: .git is a file pointing to the real git dir
    if git_path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&git_path) {
            if let Some(dir) = content.strip_prefix("gitdir: ") {
                let dir = dir.trim();
                let p = if Path::new(dir).is_absolute() {
                    PathBuf::from(dir)
                } else {
                    root.join(dir)
                };
                if p.is_dir() {
                    return Some(p);
                }
            }
        }
    }
    None
}

/// Filter out irrelevant events (e.g., .git/objects writes during gc).
fn is_relevant(event: &notify::Event) -> bool {
    use notify::EventKind;
    match &event.kind {
        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
            // Ignore .git/objects and .git/logs (noisy, not status-relevant)
            !event.paths.iter().any(|p| {
                let s = p.to_string_lossy();
                s.contains(".git/objects") || s.contains(".git/logs")
            })
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn new_returns_none_without_git() {
        let dir = tempfile::tempdir().unwrap();
        assert!(GitWatcher::new(dir.path()).is_none());
    }

    #[test]
    fn new_returns_some_with_git() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        let watcher = GitWatcher::new(dir.path());
        assert!(watcher.is_some());
    }

    #[test]
    fn has_changes_returns_false_initially() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        let watcher = GitWatcher::new(dir.path()).unwrap();
        // No changes yet
        assert!(!watcher.has_changes());
    }

    #[test]
    fn signal_change_sets_flag() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        let watcher = GitWatcher::new(dir.path()).unwrap();
        watcher.signal_change();
        assert!(watcher.has_changes());
        // Second call should be false (flag reset)
        assert!(!watcher.has_changes());
    }

    #[test]
    fn detects_file_change() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        fs::write(dir.path().join("test.txt"), "hello").unwrap();
        let watcher = GitWatcher::new(dir.path()).unwrap();
        // Clear any initial events
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = watcher.has_changes();

        // Modify a file
        fs::write(dir.path().join("test.txt"), "world").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));

        assert!(watcher.has_changes());
    }
}
