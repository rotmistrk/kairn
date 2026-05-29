//! Scenario tests for StructuredView inline editing — cancel, filter, key edit.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_struct(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

#[test]
fn edit_value_cancel_preserves_original() {
    let json = r#"{"name":"original"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::new(dir.path());
    open_struct(&mut h);
    // Navigate to "name" scalar
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value column
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Enter to edit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Type something
    h.inject_str("CHANGED");
    h.run_cycles(1);
    // Cancel with Esc
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    // Original value should remain
    assert!(
        h.content_contains("original"),
        "value should still be 'original' after cancel"
    );
}

#[test]
fn edit_key_in_dict() {
    let json = r#"{"oldkey":"value"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::new(dir.path());
    open_struct(&mut h);
    // Navigate to "oldkey" node
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Key column is default focus, press Enter to edit key
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Clear and type new key
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    for _ in 0..6 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("newkey");
    h.run_cycles(1);
    // Commit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("newkey"), "key should be updated to 'newkey'");
}

#[test]
fn filter_shows_matching_nodes() {
    let json = r#"{"alpha":"1","beta":"2","gamma":"3"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::new(dir.path());
    open_struct(&mut h);
    // Press 'f' to start filter
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    // Type "bet" to filter
    h.inject_str("bet");
    h.run_cycles(1);
    // Commit filter
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("beta"), "beta should be visible");
    assert!(!h.content_contains("alpha"), "alpha should be filtered out");
}

#[test]
fn filter_clear_restores_all() {
    let json = r#"{"alpha":"1","beta":"2"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::new(dir.path());
    open_struct(&mut h);
    // Filter for "alpha"
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("alpha");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    // Clear filter with 'F'
    h.inject_key(KeyCode::Char('F'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("alpha"), "alpha visible after clear");
    assert!(h.content_contains("beta"), "beta visible after clear");
}

#[test]
fn undo_restores_previous_value() {
    let json = r#"{"x":"before"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::new(dir.path());
    open_struct(&mut h);
    // Navigate to scalar
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Edit
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
    h.run_cycles(2);
    assert!(h.content_contains("after"), "value should be 'after'");
    // Undo
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("before"), "value should be 'before' after undo");
}
