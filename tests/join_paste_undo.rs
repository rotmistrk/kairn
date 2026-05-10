// === Backspace joining lines, J, paste undo ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn backspace_at_col0_joins_with_previous_line() {
    let dir = temp_project(&[("t.txt", "hello\nworld")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Move to line 2, enter insert mode, backspace at col 0
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Backspace, KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("helloworld"), "backspace at col 0 should join lines");
}

#[test]
fn j_command_joins_lines_with_space() {
    let dir = temp_project(&[("t.txt", "hello\nworld")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // J joins current line with next
    h.inject_key(KeyCode::Char('J'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("hello world"), "J should join with space");
}

#[test]
fn j_join_is_undoable() {
    let dir = temp_project(&[("t.txt", "hello\nworld")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('J'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("hello world"));
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.contains("hello world"));
    assert!(h.contains("hello"));
    assert!(h.contains("world"));
}

#[test]
fn paste_is_undoable() {
    let dir = temp_project(&[("t.txt", "line1\nline2")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // yy then p
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    assert_eq!(screen.matches("line1").count(), 2);
    // Undo should remove the pasted line
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    assert_eq!(screen.matches("line1").count(), 1);
}
