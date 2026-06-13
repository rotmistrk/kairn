//! Test: :log renders git commit history on screen.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_GIT_LOG;

/// Create a git repo with two commits.
fn git_project_two_commits() -> tempfile::TempDir {
    let dir = temp_project(&[("a.txt", "initial\n")]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();

    // First commit
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("a.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let oid1 = repo
        .commit(Some("HEAD"), &sig, &sig, "first commit", &tree, &[])
        .unwrap();

    // Second commit
    std::fs::write(dir.path().join("a.txt"), "updated\n").unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("a.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let parent = repo.find_commit(oid1).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "second commit", &tree, &[&parent])
        .unwrap();
    dir
}

#[test]
fn git_log_renders_commit_messages() {
    let dir = git_project_two_commits();
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Dispatch CM_GIT_LOG (same as M-x log)
    h.dispatch_command(CM_GIT_LOG, None);
    h.run_cycles(2);

    // Give async log loading time to finish
    std::thread::sleep(std::time::Duration::from_millis(100));
    h.run_cycles(5);

    // Verify commit messages appear on screen
    assert!(
        h.contains("second commit"),
        "git log should show 'second commit', got:\n{}",
        h.screen_text()
    );
    assert!(
        h.contains("first commit"),
        "git log should show 'first commit', got:\n{}",
        h.screen_text()
    );
}
