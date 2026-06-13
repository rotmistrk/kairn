//! Diff -U context rendering — verifies fold markers with custom context.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

/// Create a git project with initial commit, then modify the file.
fn git_project_modified(filename: &str, initial: &str, modified: &str) -> tempfile::TempDir {
    let dir = temp_project(&[(filename, initial)]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    std::fs::write(dir.path().join(filename), modified).unwrap();
    dir
}

fn open_file(h: &mut TestHarness, name: &str) {
    let path = h.state.root_dir().join(name);
    let req = OpenFileRequest::new(path);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);
}

fn send_ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    for ch in cmd.chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(5);
}

#[test]
fn diff_u1_shows_fold_markers() {
    // 20 lines, modify line 10 — with -U1, most context should be folded
    let initial: String = (1..=20).map(|i| format!("line {i}\n")).collect();
    let mut modified = initial.clone();
    modified = modified.replace("line 10\n", "CHANGED\n");
    let dir = git_project_modified("big.rs", &initial, &modified);
    let mut h = TestHarness::with_size(dir.path(), 80, 30);
    open_file(&mut h, "big.rs");
    send_ex(&mut h, "diff -U1");

    let text = h.screen_text();
    // With -U1, there should be fold markers ("--- N lines ---")
    assert!(
        text.contains("lines ---"),
        "fold markers should appear with -U1. Screen:\n{text}"
    );
    // The changed content should be visible
    assert!(
        text.contains("CHANGED"),
        "changed line should be visible. Screen:\n{text}"
    );
}

#[test]
fn diff_u1_fewer_context_than_default() {
    // With U1 we expect fewer visible context lines than default (3)
    let initial: String = (1..=20).map(|i| format!("line {i}\n")).collect();
    let mut modified = initial.clone();
    modified = modified.replace("line 10\n", "CHANGED\n");
    let dir = git_project_modified("ctx.rs", &initial, &modified);
    let mut h = TestHarness::with_size(dir.path(), 80, 30);
    open_file(&mut h, "ctx.rs");
    send_ex(&mut h, "diff -U1");

    let text_u1 = h.screen_text();
    // "line 8" is 2 lines before the change — with U1 it should be folded
    assert!(
        !text_u1.contains("line 8"),
        "line 8 should be folded away with -U1. Screen:\n{text_u1}"
    );
}
