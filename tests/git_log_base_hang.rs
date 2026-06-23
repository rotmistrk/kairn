//! Reproduce: pressing 'b' in git log should not hang.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_GIT_LOG;
use txv_core::prelude::*;

fn git_project_two_commits() -> tempfile::TempDir {
    let dir = temp_project(&[("a.txt", "initial\n")]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();

    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("a.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let oid1 = repo
        .commit(Some("HEAD"), &sig, &sig, "first commit", &tree, &[])
        .unwrap();

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
fn pressing_b_in_log_does_not_hang() {
    let dir = git_project_two_commits();
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open log
    h.dispatch_command(CM_GIT_LOG, None);
    h.run_cycles(2);
    std::thread::sleep(std::time::Duration::from_millis(100));
    h.run_cycles(5);

    // Press 'b' to set diff base
    h.inject_key(KeyCode::Char('b'), KeyMod::default());
    // Run many cycles — must not hang
    h.run_cycles(60);

    // If we get here, no hang. Verify something rendered.
    assert!(h.screen_text().len() > 0, "screen should have content after pressing b");
}
