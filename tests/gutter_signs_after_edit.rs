//! Scenario: git gutter signs appear after inserting a line in a tracked file.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

/// Create git project with committed file.
fn git_project_committed(filename: &str, content: &str) -> tempfile::TempDir {
    let dir = temp_project(&[(filename, content)]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
    dir
}

#[test]
fn gutter_sign_appears_after_insert() {
    let dir = git_project_committed("hello.txt", "line1\nline2\nline3\n");
    let mut h = TestHarness::with_size(dir.path(), 60, 10);
    h.run_cycles(1);

    // Open the committed file
    let req = OpenFileRequest::new(dir.path().join("hello.txt"));
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Enter insert mode and add a new line
    h.inject_key(KeyCode::Char('o'), KeyMod::NONE); // open line below
    h.run_cycles(2);
    h.inject_str("new line");
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::NONE); // back to normal
    h.run_cycles(5);

    // The gutter should show ▎ marker (Added sign) on the new line
    assert!(
        h.content_contains("▎"),
        "git gutter sign ▎ should appear after inserting a line"
    );
}

#[test]
fn gutter_signs_clear_after_commit() {
    let dir = git_project_committed("f.txt", "line1\nline2\n");
    let mut h = TestHarness::with_size(dir.path(), 60, 10);
    h.run_cycles(1);

    let req = OpenFileRequest::new(dir.path().join("f.txt"));
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Insert a line
    h.inject_key(KeyCode::Char('o'), KeyMod::NONE);
    h.run_cycles(2);
    h.inject_str("added");
    h.inject_key(KeyCode::Esc, KeyMod::NONE);
    h.run_cycles(5);
    assert!(h.content_contains("▎"), "sign should appear after insert");

    // Save the file
    h.inject_key(KeyCode::Char(':'), KeyMod::NONE);
    h.inject_str("w");
    h.inject_key(KeyCode::Enter, KeyMod::NONE);
    h.run_cycles(3);

    // Commit via git2
    let repo = git2::Repository::open(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("f.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    let head = repo.head().unwrap().peel_to_commit().unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "add line", &tree, &[&head])
        .unwrap();

    // Save again to trigger gutter sign refresh (save calls refresh_gutter_signs)
    h.inject_key(KeyCode::Char(':'), KeyMod::NONE);
    h.inject_str("w");
    h.inject_key(KeyCode::Enter, KeyMod::NONE);
    h.run_cycles(5);

    assert!(
        !h.content_contains("▎"),
        "gutter sign ▎ should be gone after commit (working tree matches HEAD)"
    );
}
