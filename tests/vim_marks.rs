// === Vim marks: m<letter> to set, '<letter> to jump ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, none());
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

#[test]
fn set_and_jump_to_mark() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3\nline4\nline5")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);

    // Move to line 3 (2 x j)
    h.inject_key(KeyCode::Char('j'), none());
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);

    // Set mark 'a' at line 3
    h.inject_key(KeyCode::Char('m'), none());
    h.inject_key(KeyCode::Char('a'), none());
    h.run_cycles(1);

    // Move to line 5
    h.inject_key(KeyCode::Char('j'), none());
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);

    // Jump back to mark 'a'
    h.inject_key(KeyCode::Char('\''), none());
    h.inject_key(KeyCode::Char('a'), none());
    h.run_cycles(1);

    // Verify cursor is on line 3 (0-indexed line 2)
    let pos = helpers::cursor_at(&h).expect("cursor should be visible");
    assert_eq!(pos.0, 2, "cursor should be on line 3 (index 2) after 'a jump");
}

#[test]
fn jump_to_unset_mark_shows_error() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);

    // Try jumping to unset mark 'z'
    h.inject_key(KeyCode::Char('\''), none());
    h.inject_key(KeyCode::Char('z'), none());
    h.run_cycles(1);

    // Should show error message (not crash)
    // Cursor should remain at line 0
    let pos = helpers::cursor_at(&h).expect("cursor should be visible");
    assert_eq!(pos.0, 0, "cursor should stay at line 0 on unset mark");
}
