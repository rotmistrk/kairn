// === :!cmd error handling and filter undo ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn bang_nonexistent_command_does_not_crash() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    assert!(h.contains("hello"));
    // Run a command that doesn't exist — sh -c will produce stderr
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("!nonexistent_cmd_xyz_999\n");
    h.run_cycles(1);
    // Should not crash — a new output tab opens (may be empty)
    // The app is still running and rendering
    let screen = h.screen_text();
    assert!(!screen.is_empty(), "app should still render after failed command");
}

#[test]
fn filter_sort_is_undoable() {
    let dir = temp_project(&[("t.txt", "cherry\napple\nbanana")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Sort all lines
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("%!sort\n");
    h.run_cycles(1);
    // Verify sorted
    let screen = h.screen_text();
    let apple_pos = screen.find("apple").unwrap_or(999);
    let banana_pos = screen.find("banana").unwrap_or(999);
    assert!(apple_pos < banana_pos, "should be sorted");
    // Undo should restore original order
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    let cherry_pos = screen.find("cherry").unwrap_or(999);
    let apple_pos = screen.find("apple").unwrap_or(999);
    assert!(cherry_pos < apple_pos, "undo should restore original order");
}

#[test]
fn bang_command_with_stderr_still_works() {
    let dir = temp_project(&[("t.txt", "data")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Command that writes to stderr but still produces stdout
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("!echo OUTPUT; echo ERR >&2\n");
    h.run_cycles(1);
    // stdout should appear
    assert!(h.contains("OUTPUT"));
}
