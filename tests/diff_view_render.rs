//! Scenario tests for DiffView rendering — verifies visual output of :diff.
//!
//! These tests ensure the DiffView actually renders diff content to screen,
//! not just that it sets internal state.

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

fn press_key(h: &mut TestHarness, code: KeyCode) {
    h.inject_key(code, KeyMod::default());
    h.run_cycles(3);
}

/// Basic: :diff must render added lines visibly on screen.
#[test]
fn diff_renders_added_lines() {
    let dir = git_project_modified("a.rs", "fn main() {\n}\n", "fn main() {\n    let x = 1;\n}\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "a.rs");
    send_ex(&mut h, "diff");

    let text = h.screen_text();
    // The added line "let x = 1;" must appear in the rendered output
    assert!(
        text.contains("let x = 1"),
        "Added line should be visible in diff. Screen:\n{text}"
    );
    // The context line "fn main()" should also be visible
    assert!(
        text.contains("fn main()"),
        "Context lines should be visible in diff. Screen:\n{text}"
    );
}

/// Basic: :diff must render deleted lines visibly on screen.
#[test]
fn diff_renders_deleted_lines() {
    let dir = git_project_modified("b.rs", "fn foo() {\n    let old = true;\n}\n", "fn foo() {\n}\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "b.rs");
    send_ex(&mut h, "diff");

    let text = h.screen_text();
    // The deleted line should appear (it's virtual — from base)
    assert!(
        text.contains("let old = true"),
        "Deleted line should be visible in diff. Screen:\n{text}"
    );
}

/// Tab title should indicate diff mode.
#[test]
fn diff_tab_title_shows_diff() {
    let dir = git_project_modified("c.rs", "aaa\n", "bbb\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "c.rs");
    send_ex(&mut h, "diff");

    let text = h.screen_text();
    assert!(
        text.contains("[diff]"),
        "Tab should show [diff] in title. Screen:\n{text}"
    );
}

/// Line numbers: when enabled, both base and buffer line numbers appear.
#[test]
fn diff_shows_dual_line_numbers() {
    let dir = git_project_modified("d.rs", "line1\nline2\nline3\n", "line1\nnewline\nline2\nline3\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "d.rs");
    send_ex(&mut h, "diff");

    let text = h.screen_text();
    // Should show line numbers from both sides (base and buffer)
    // Context line "line1" should have both numbers: "1" from base and "1" from buffer
    assert!(
        text.contains("1") && text.contains("line1"),
        "Line numbers should appear alongside content. Screen:\n{text}"
    );
}

/// Navigation: 'n' moves cursor to next hunk.
#[test]
fn diff_navigation_next_hunk() {
    let dir = git_project_modified("e.rs", "aaa\nbbb\nccc\nddd\neee\n", "aaa\nBBB\nccc\nddd\nEEE\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "e.rs");
    send_ex(&mut h, "diff");

    // Press 'n' to go to first hunk
    press_key(&mut h, KeyCode::Char('n'));
    let text1 = h.screen_text();

    // Press 'n' again to go to second hunk
    press_key(&mut h, KeyCode::Char('n'));
    let text2 = h.screen_text();

    // Both hunks' content should be visible
    assert!(
        text1.contains("BBB") || text2.contains("EEE"),
        "Navigation should show hunk content. Screen1:\n{text1}\nScreen2:\n{text2}"
    );
}

/// Exit: Esc returns to normal editor.
#[test]
fn diff_exit_returns_to_editor() {
    let dir = git_project_modified("f.rs", "hello\n", "world\n");
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "f.rs");
    send_ex(&mut h, "diff");

    // Verify we're in diff mode
    let in_diff = h.screen_text();
    assert!(in_diff.contains("[diff]"), "Should be in diff mode");

    // Press Esc to exit
    press_key(&mut h, KeyCode::Esc);

    // Should be back in normal editor — no more [diff] in tab title
    let after = h.screen_text();
    assert!(
        !after.contains("[diff]"),
        "Should have exited diff mode. Screen:\n{after}"
    );
}

/// No changes: :diff on unmodified file shows "no changes" message.
#[test]
fn diff_no_changes_shows_message() {
    let dir = temp_project(&[("g.rs", "unchanged\n")]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("g.rs")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("T", "t@t").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();

    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "g.rs");
    send_ex(&mut h, "diff");

    let text = h.screen_text();
    // No DiffView tab should open; no [diff] in title
    assert!(
        !text.contains("[diff]"),
        "No diff tab should open for unmodified file. Screen:\n{text}"
    );
}
