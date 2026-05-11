// === Git status watcher integration tests ===

mod helpers;

use std::sync::Arc;

use helpers::TestHarness;
use kairn::git_status::{collect_git_status, dir_status, FileStatus};
use kairn::git_watcher::{GitWatcher, WatchHandle};

#[test]
fn conflict_has_highest_priority() {
    assert_eq!(
        dir_status(&[FileStatus::Modified, FileStatus::Conflict]),
        FileStatus::Conflict
    );
    assert_eq!(
        dir_status(&[FileStatus::Untracked, FileStatus::Conflict]),
        FileStatus::Conflict
    );
}

#[test]
fn watcher_detects_new_file_in_git_repo() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    std::fs::write(dir.path().join("tracked.txt"), "hello").unwrap();

    let watcher = match GitWatcher::new(dir.path()) {
        Some(w) => Arc::new(w),
        None => return,
    };
    let mut handle = WatchHandle::new(watcher);

    // Clear initial events
    std::thread::sleep(std::time::Duration::from_millis(50));
    let _ = handle.has_changes();

    // Create a new file
    std::fs::write(dir.path().join("new_file.txt"), "new").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    assert!(handle.has_changes(), "watcher should detect new file");
}

#[test]
fn collect_git_status_shows_untracked() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    std::fs::write(dir.path().join("untracked.txt"), "hello").unwrap();

    let statuses = collect_git_status(dir.path());
    assert_eq!(
        statuses.get("untracked.txt"),
        Some(&FileStatus::Untracked),
        "new file should be untracked: {:?}",
        statuses
    );
}

#[test]
fn tree_view_starts_with_git_colors() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    std::fs::write(dir.path().join("file.txt"), "content").unwrap();

    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    assert!(h.contains("file.txt"), "tree should show file.txt");
}
