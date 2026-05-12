mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn undo_edit_value() {
    let json = r#"{"name":"original"}"#;
    let dir = temp_project(&[("undo.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "name" scalar
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value column
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Edit value
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    for _ in 0..8 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("changed");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("changed"), "value should be 'changed' after edit");
    // Undo
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("original"), "value should be 'original' after undo");
}

#[test]
fn undo_delete() {
    let json = r#"{"a":1,"b":2}"#;
    let dir = temp_project(&[("undodel.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "a"
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Delete
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.content_contains("\"a\""), "'a' should be gone after delete");
    // Undo
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("a"), "'a' should be back after undo");
}

#[test]
fn redo_after_undo() {
    let json = r#"{"x":"before"}"#;
    let dir = temp_project(&[("redo.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "x" scalar
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value column
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Edit value
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    for _ in 0..6 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("after");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("after"), "value should be 'after'");
    // Undo
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("before"), "value should be 'before' after undo");
    // Redo
    h.inject_key(
        KeyCode::Char('r'),
        KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    );
    h.run_cycles(1);
    assert!(h.content_contains("after"), "value should be 'after' after redo");
}

#[test]
fn undo_clears_on_new_edit() {
    let json = r#"[10,20,30]"#;
    let dir = temp_project(&[("undoclear.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to first element
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Delete first element (10)
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    // Undo — 10 should be back
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("10"), "10 should be back after undo");
    // Make a different edit — delete again (this should clear redo)
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    // Redo should do nothing (redo history cleared)
    h.inject_key(
        KeyCode::Char('r'),
        KeyMod {
            ctrl: true,
            alt: false,
            shift: false,
        },
    );
    h.run_cycles(1);
    // The array should still show [2] (one element was deleted)
    assert!(h.content_contains("[2]"), "redo should have no effect after new edit");
}
