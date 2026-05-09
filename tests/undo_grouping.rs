// === Feature 1: Undo grouping for insert sessions ===
// All edits during a single insert session should undo as ONE operation.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn insert_session_undoes_as_one_group() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Enter insert mode, type multiple chars, exit
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_str("ABC");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("ABChello"));
    // Single undo should remove ALL inserted chars at once
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    // After one undo, buffer should be back to original "hello"
    // (not "ABhello" which would mean only last char was undone)
    assert!(
        h.contains("hello") && !h.contains("A") && !h.contains("B"),
        "single undo should remove entire insert session"
    );
}

#[test]
fn multiple_insert_sessions_undo_separately() {
    let dir = temp_project(&[("t.txt", "base")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // First insert session: 'A' appends after cursor (col 0)
    h.inject_key(KeyCode::Char('A'), KeyMod::default());
    h.inject_str("11");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("base11"));
    // Second insert session: 'A' appends at end again
    h.inject_key(KeyCode::Char('A'), KeyMod::default());
    h.inject_str("22");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("base1122"));
    // First undo removes second session
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("base11"));
    assert!(!h.contains("22"));
    // Second undo removes first session
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("base"));
    assert!(!h.contains("11"));
}
