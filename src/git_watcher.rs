//! GitWatcher — reactive file watching for git status changes.
//! Uses notify crate (inotify/kqueue) to detect changes without polling.
//! Supports multiple consumers via generation counter.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use notify::{RecommendedWatcher, RecursiveMode, Watcher};

/// Shared generation counter — incremented on each relevant change.
pub struct GitWatcher {
    generation: Arc<AtomicU64>,
    _watcher: RecommendedWatcher,
}

impl GitWatcher {
    /// Create a watcher for the given project root.
    /// Returns None if watcher creation fails.
    pub fn new(root: &Path) -> Option<Self> {
        let git_dir = find_git_dir(root)?;
        let generation = Arc::new(AtomicU64::new(1));
        let gen = generation.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
            if let Ok(event) = res {
                if is_relevant(&event) {
                    gen.fetch_add(1, Ordering::Relaxed);
                }
            }
        })
        .ok()?;

        // Only watch git internals (cheap, few FDs)
        let _ = watcher.watch(&git_dir.join("index"), RecursiveMode::NonRecursive);
        let _ = watcher.watch(&git_dir.join("refs"), RecursiveMode::NonRecursive);
        // Watch .gitignore for rule changes
        let gitignore = root.join(".gitignore");
        if gitignore.exists() {
            let _ = watcher.watch(&gitignore, RecursiveMode::NonRecursive);
        }

        Some(Self {
            generation,
            _watcher: watcher,
        })
    }

    /// Current generation number.
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Force a generation bump (e.g., on CM_SAVE).
    pub fn signal_change(&self) {
        self.generation.fetch_add(1, Ordering::Relaxed);
    }
}

/// Per-consumer handle that tracks last-seen generation.
pub struct WatchHandle {
    watcher: Arc<GitWatcher>,
    last_gen: u64,
}

impl WatchHandle {
    pub fn new(watcher: Arc<GitWatcher>) -> Self {
        let last_gen = watcher.generation();
        Self { watcher, last_gen }
    }

    /// Returns true if changes occurred since last check. Updates internal state.
    pub fn has_changes(&mut self) -> bool {
        let current = self.watcher.generation();
        if current != self.last_gen {
            self.last_gen = current;
            return true;
        }
        false
    }

    /// Signal a change on the underlying watcher.
    pub fn signal_change(&self) {
        self.watcher.signal_change();
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
    fn handle_returns_false_initially() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        let watcher = Arc::new(GitWatcher::new(dir.path()).unwrap());
        let mut handle = WatchHandle::new(watcher);
        assert!(!handle.has_changes());
    }

    #[test]
    fn signal_change_detected_by_handle() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        let watcher = Arc::new(GitWatcher::new(dir.path()).unwrap());
        let mut handle = WatchHandle::new(watcher.clone());
        watcher.signal_change();
        assert!(handle.has_changes());
        // Second call should be false (no new changes)
        assert!(!handle.has_changes());
    }

    #[test]
    #[test]
    fn multiple_handles_see_same_change() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir(dir.path().join(".git")).unwrap();
        fs::write(dir.path().join(".git/index"), "").unwrap();
        let watcher = Arc::new(GitWatcher::new(dir.path()).unwrap());
        let mut h1 = WatchHandle::new(watcher.clone());
        let mut h2 = WatchHandle::new(watcher.clone());
        watcher.signal_change();
        assert!(h1.has_changes());
        assert!(h2.has_changes());
    }
}
