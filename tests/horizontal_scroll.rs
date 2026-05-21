//! Tests for horizontal scrolling in nowrap mode.

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

/// In nowrap mode, moving cursor past right edge scrolls horizontally.
#[test]
fn cursor_right_scrolls_view() {
    let long_line = format!("START{}END\n", "M".repeat(80));
    let dir = temp_project(&[("f.txt", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    // Cursor at col 0 — "START" visible
    assert!(h.content_contains("START"));

    // Move to end of line
    h.inject_key(KeyCode::Char('$'), none());
    h.run_cycles(2);

    // "END" should now be visible, "START" scrolled off
    assert!(h.content_contains("END"));
    assert!(!h.content_contains("START"));
}

/// Moving cursor back to start scrolls view left.
#[test]
fn cursor_left_scrolls_back() {
    let long_line = format!("START{}END\n", "M".repeat(80));
    let dir = temp_project(&[("f.txt", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    // Move to end then back to start
    h.inject_key(KeyCode::Char('$'), none());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('0'), none());
    h.run_cycles(2);

    // "START" visible again
    assert!(h.content_contains("START"));
}

/// In wrap mode, h_scroll stays at 0 (no horizontal scrolling).
#[test]
fn wrap_mode_no_horizontal_scroll() {
    let long_line = format!("START{}END\n", "M".repeat(80));
    let dir = temp_project(&[("f.txt", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set wrap");

    // Move to end — in wrap mode, line wraps so START is still visible
    h.inject_key(KeyCode::Char('$'), none());
    h.run_cycles(2);

    assert!(h.content_contains("START"));
}
