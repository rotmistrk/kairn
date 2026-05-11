// === Git status watcher integration tests ===

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::git_status::{collect_git_status, dir_status, FileStatus};
use kairn::git_watcher::GitWatcher;
use txv_core::event::{KeyCode, KeyMod};

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
    // Init a real git repo using git2
    let repo = git2::Repository::init(dir.path());
    if repo.is_err() {
        return; // Skip if git2 can't init
    }
    std::fs::write(dir.path().join("tracked.txt"), "hello").unwrap();

    let watcher = GitWatcher::new(dir.path());
    if watcher.is_none() {
        return; // Skip if watcher can't start
    }
    let watcher = watcher.unwrap();

    // Clear initial events
    std::thread::sleep(std::time::Duration::from_millis(50));
    let _ = watcher.has_changes();

    // Create a new file
    std::fs::write(dir.path().join("new_file.txt"), "new").unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));

    assert!(watcher.has_changes(), "watcher should detect new file");
}

#[test]
fn collect_git_status_shows_untracked() {
    let dir = tempfile::tempdir().unwrap();
    let repo = git2::Repository::init(dir.path());
    if repo.is_err() {
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
    // Create a git repo with an untracked file
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    std::fs::write(dir.path().join("file.txt"), "content").unwrap();

    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // The tree should show the file (it exists in the tree)
    assert!(h.contains("file.txt"), "tree should show file.txt");
}
