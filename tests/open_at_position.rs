// === Open file at specific line/col (goto_definition, next-error) ===

mod helpers;

use helpers::{cursor_at, temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

fn focus_center(h: &mut TestHarness) {
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
}

#[test]
fn open_file_at_line_col_positions_cursor() {
    let dir = temp_project(&[("multi.txt", "line one\nline two\nline three\nline four\nline five")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let path = dir.path().join("multi.txt");
    let req = OpenFileRequest::at(path, 2, 5);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(1);
    focus_center(&mut h);

    assert!(h.content_contains("line three"), "file should be open");
    let pos = cursor_at(&h);
    assert_eq!(pos, Some((2, 5)), "cursor should be at line 2, col 5");
}

#[test]
fn open_file_without_line_col_starts_at_top() {
    let dir = temp_project(&[("hello.txt", "first\nsecond\nthird")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let path = dir.path().join("hello.txt");
    let req = OpenFileRequest::new(path);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(1);
    focus_center(&mut h);

    let pos = cursor_at(&h);
    assert_eq!(pos, Some((0, 0)), "cursor should be at line 0, col 0");
}

#[test]
fn open_file_at_line_beyond_end_clamps() {
    let dir = temp_project(&[("short.txt", "one\ntwo")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let path = dir.path().join("short.txt");
    let req = OpenFileRequest::at(path, 99, 0);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(1);
    focus_center(&mut h);

    let pos = cursor_at(&h);
    assert_eq!(pos, Some((1, 0)), "cursor should clamp to last line");
}
