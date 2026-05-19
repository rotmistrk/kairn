//! Integration tests: ex command ranges (., %, n,m, .,$, 1,n, '<,'>).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_and_focus(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

fn ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), none());
    h.run_cycles(1);
    h.inject_str(cmd);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);
}

// --- Range: % (all lines) ---

#[test]
fn range_percent_delete_removes_all() {
    let dir = temp_project(&[("f.txt", "aaa\nbbb\nccc\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    ex(&mut h, "%d");

    assert!(!h.content_contains("aaa"));
    assert!(!h.content_contains("bbb"));
    assert!(!h.content_contains("ccc"));
}

#[test]
fn range_percent_substitute() {
    let dir = temp_project(&[("f.txt", "foo\nfoo\nfoo\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    ex(&mut h, "%s/foo/bar/g");

    assert!(!h.content_contains("foo"));
    assert!(h.content_contains("bar"));
}

// --- Range: n,m (explicit line numbers) ---

#[test]
fn range_explicit_delete_lines_2_3() {
    let dir = temp_project(&[("f.txt", "line1\nline2\nline3\nline4\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    ex(&mut h, "2,3d");

    assert!(h.content_contains("line1"));
    assert!(!h.content_contains("line2"));
    assert!(!h.content_contains("line3"));
    assert!(h.content_contains("line4"));
}

#[test]
fn range_explicit_substitute() {
    let dir = temp_project(&[("f.txt", "aXa\nbXb\ncXc\ndXd\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    ex(&mut h, "2,3s/X/Y/g");

    assert!(h.content_contains("aXa")); // line 1 unchanged
    assert!(h.content_contains("bYb")); // line 2 changed
    assert!(h.content_contains("cYc")); // line 3 changed
    assert!(h.content_contains("dXd")); // line 4 unchanged
}

// --- Range: . (current line) ---

#[test]
fn range_dot_delete_current_line() {
    let dir = temp_project(&[("f.txt", "keep\ndelete_me\nkeep2\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    // Move to line 2
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);

    ex(&mut h, ".d");

    assert!(h.content_contains("keep"));
    assert!(!h.content_contains("delete_me"));
    assert!(h.content_contains("keep2"));
}

// --- Range: .,$ (current to end) ---

#[test]
fn range_dot_dollar_deletes_to_end() {
    let dir = temp_project(&[("f.txt", "first\nsecond\nthird\nfourth\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    // Move to line 3
    h.inject_str("2j");
    h.run_cycles(1);

    ex(&mut h, ".,$d");

    assert!(h.content_contains("first"));
    assert!(h.content_contains("second"));
    assert!(!h.content_contains("third"));
    assert!(!h.content_contains("fourth"));
}

// --- Range: 1,. (start to current) ---

#[test]
fn range_1_dot_deletes_from_start() {
    let dir = temp_project(&[("f.txt", "aa\nbb\ncc\ndd\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    // Move to line 2
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);

    ex(&mut h, "1,.d");

    assert!(!h.content_contains("aa"));
    assert!(!h.content_contains("bb"));
    assert!(h.content_contains("cc"));
}

// --- Range: yank ---

#[test]
fn range_yank_and_paste() {
    let dir = temp_project(&[("f.txt", "alpha\nbeta\ngamma\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    ex(&mut h, "1,2y");

    // Move to end and paste
    h.inject_key(KeyCode::Char('G'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('p'), none());
    h.run_cycles(2);

    let screen = h.screen_text();
    assert!(screen.matches("alpha").count() >= 2);
}
