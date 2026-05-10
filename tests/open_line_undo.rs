// === o/O undo grouping ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn o_open_line_and_type_undoes_as_one() {
    let dir = temp_project(&[("t.txt", "first\nsecond")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // o opens line below, type text, Esc
    h.inject_key(KeyCode::Char('o'), KeyMod::default());
    h.inject_str("new line");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("new line"));
    // Single undo should remove the new line AND its content
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.contains("new line"), "undo should remove entire o session");
    assert!(h.contains("first"));
    assert!(h.contains("second"));
}

#[test]
fn big_o_open_above_undoes_as_one() {
    let dir = temp_project(&[("t.txt", "first\nsecond")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // O opens line above, type text, Esc
    h.inject_key(KeyCode::Char('O'), KeyMod::default());
    h.inject_str("above");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("above"));
    // Single undo
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.contains("above"), "undo should remove entire O session");
}
