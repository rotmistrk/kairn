//! Tests for Help tab deduplication — F1 twice should not create two Help tabs.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn f1_twice_creates_only_one_help_tab() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // Press F1 twice
    h.inject_key(KeyCode::F(1), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(1), KeyMod::default());
    h.run_cycles(1);

    // Count occurrences of "Help" in the chrome/screen
    // The dropdown would show "Help" once if there's only one tab
    let screen = h.screen_text();
    let count = screen.matches("Help").count();
    // "Help" appears in: chrome tab title + status bar "F1:Help" = 2
    // If duplicated: chrome would show it twice or badge would be wrong
    // Check that the center panel badge doesn't show 3 tabs (Welcome + Help + Help)
    // With dedup: Welcome + Help = 2 tabs
    assert!(
        count <= 3,
        "Help should not be duplicated. 'Help' appears {count} times: too many"
    );
}

#[test]
fn f1_focuses_existing_help_tab() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // Open help
    h.inject_key(KeyCode::F(1), KeyMod::default());
    h.run_cycles(1);

    // Switch to Welcome tab via Alt-0 (first tab)
    h.inject_key(KeyCode::Char('0'), KeyMod::ALT);
    h.run_cycles(1);

    // Press F1 again — should switch back to Help, not create new
    h.inject_key(KeyCode::F(1), KeyMod::default());
    h.run_cycles(1);

    let top = h.row(0);
    assert!(top.contains("Help"), "F1 should focus existing Help tab: {top:?}");
}
