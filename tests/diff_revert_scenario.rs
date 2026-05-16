//! Scenario tests for diff revert — full integration with git repo, harness, and key input.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

/// Create a temp project with a git repo and an initial commit.
fn git_project(filename: &str, initial_content: &str) -> tempfile::TempDir {
    let dir = temp_project(&[(filename, initial_content)]);
    let repo = match git2::Repository::init(dir.path()) {
        Ok(r) => r,
        Err(_) => return dir,
    };
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new(filename)).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[]).unwrap();
    dir
}

fn open_file(h: &mut TestHarness, name: &str) {
    let path = h.state.root_dir.join(name);
    let req = OpenFileRequest {
        path,
        line: None,
        col: None,
        diff: false,
    };
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(2);
}

fn send_ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    for ch in cmd.chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
}

fn press_key(h: &mut TestHarness, code: KeyCode) {
    h.inject_key(code, KeyMod::default());
    h.run_cycles(1);
}

#[test]
fn scenario_revert_added_line_via_hotkey() {
    let dir = git_project("main.rs", "fn main() {\n    println!(\"hello\");\n}\n");
    std::fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    let x = 1;\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "main.rs");

    send_ex(&mut h, "diff");
    press_key(&mut h, KeyCode::Char('n'));
    press_key(&mut h, KeyCode::Char('R'));

    send_ex(&mut h, "w");
    let content = std::fs::read_to_string(dir.path().join("main.rs")).unwrap();
    assert_eq!(content, "fn main() {\n    println!(\"hello\");\n}\n");
}

#[test]
fn scenario_revert_via_ex_command() {
    let dir = git_project("lib.rs", "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n");
    std::fs::write(
        dir.path().join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a.wrapping_add(b)\n}\n",
    )
    .unwrap();

    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "lib.rs");

    send_ex(&mut h, "diff");
    press_key(&mut h, KeyCode::Char('n'));
    send_ex(&mut h, "revert");

    send_ex(&mut h, "w");
    let content = std::fs::read_to_string(dir.path().join("lib.rs")).unwrap();
    assert_eq!(content, "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n");
}

#[test]
fn scenario_revert_on_context_does_not_modify() {
    let dir = git_project("a.txt", "line1\nline2\nline3\n");
    std::fs::write(dir.path().join("a.txt"), "line1\nline2\nline3\nnew\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "a.txt");

    send_ex(&mut h, "diff");
    // Don't navigate — cursor on context
    press_key(&mut h, KeyCode::Char('R'));

    send_ex(&mut h, "w");
    let content = std::fs::read_to_string(dir.path().join("a.txt")).unwrap();
    assert_eq!(content, "line1\nline2\nline3\nnew\n");
}

#[test]
fn scenario_revert_deleted_line() {
    let dir = git_project("b.txt", "aaa\nbbb\nccc\n");
    std::fs::write(dir.path().join("b.txt"), "aaa\nccc\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "b.txt");

    send_ex(&mut h, "diff");
    press_key(&mut h, KeyCode::Char('n'));
    press_key(&mut h, KeyCode::Char('R'));

    send_ex(&mut h, "w");
    let content = std::fs::read_to_string(dir.path().join("b.txt")).unwrap();
    assert_eq!(content, "aaa\nbbb\nccc\n");
}

#[test]
fn scenario_revert_abbreviation_rev() {
    let dir = git_project("c.txt", "hello\nworld\n");
    std::fs::write(dir.path().join("c.txt"), "hello\nchanged\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "c.txt");

    send_ex(&mut h, "diff");
    press_key(&mut h, KeyCode::Char('n'));
    send_ex(&mut h, "rev");

    send_ex(&mut h, "w");
    let content = std::fs::read_to_string(dir.path().join("c.txt")).unwrap();
    assert_eq!(content, "hello\nworld\n");
}
