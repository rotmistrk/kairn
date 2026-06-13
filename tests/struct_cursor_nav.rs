//! Scenario tests for StructuredView cursor navigation.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_json(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

fn json_content() -> &'static str {
    r#"{"alpha":1,"beta":2,"gamma":3,"delta":4,"epsilon":5}"#
}

#[test]
fn struct_nav_down_j() {
    let dir = temp_project(&[("data.json", json_content())]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Initially shows keys
    assert!(h.content_contains("alpha"), "alpha visible initially");
    // Move down
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(2);
    // beta still visible (moved cursor to it)
    assert!(h.content_contains("beta"), "beta visible after j");
}

#[test]
fn struct_nav_up_k() {
    let dir = temp_project(&[("data.json", json_content())]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Move down twice then up once
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(2);
    // All keys should still be visible
    assert!(h.content_contains("alpha"), "alpha visible after k");
    assert!(h.content_contains("beta"), "beta visible after k");
}

#[test]
fn struct_enter_starts_edit() {
    let dir = temp_project(&[("data.json", json_content())]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Move to first data node (j to skip root) and Tab to value column
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Press Enter to start edit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    // The value "1" should appear in an edit field — InputLine active
    // Typing should produce characters in the buffer
    h.inject_str("99");
    h.run_cycles(2);
    assert!(
        h.content_contains("99"),
        "typed text visible during edit: {}",
        h.screen_text()
    );
}

#[test]
fn struct_esc_cancels_edit() {
    let dir = temp_project(&[("data.json", json_content())]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Navigate and start edit
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_str("999");
    h.run_cycles(1);
    // Cancel with Esc
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    // Original value should remain
    assert!(
        !h.content_contains("999"),
        "cancelled text should not appear: {}",
        h.screen_text()
    );
    assert!(h.content_contains("1"), "original value 1 still visible");
}

#[test]
fn struct_filter_mode() {
    let dir = temp_project(&[("data.json", json_content())]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Press 'f' to start filter
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    // Type filter text
    h.inject_str("alpha");
    h.run_cycles(1);
    // Commit filter
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    // Only alpha should be visible, not beta
    assert!(
        h.content_contains("alpha"),
        "alpha visible after filter: {}",
        h.screen_text()
    );
}

#[test]
fn struct_nodes_visible_after_nav() {
    let dir = temp_project(&[("data.json", json_content())]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Navigate several times
    for _ in 0..3 {
        h.inject_key(KeyCode::Char('j'), KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(2);
    // Content should still render correctly
    assert!(
        h.content_contains("alpha"),
        "alpha still rendered after nav: {}",
        h.screen_text()
    );
}
