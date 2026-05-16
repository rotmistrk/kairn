//! Tests for tab title truncation (task 010).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn short_title_not_truncated() {
    let dir = temp_project(&[("hello.rs", "fn main() {}")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);
    // Open the file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);

    // "hello.rs" should appear fully in chrome
    assert!(h.content_contains("hello.rs"), "short title should not be truncated");
}

#[test]
fn long_title_gets_truncated_with_ellipsis() {
    // Create a file with a very long name
    let long_name = format!("{}.rs", "a".repeat(70));
    let dir = temp_project(&[(&long_name, "fn main() {}")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);
    // Open the file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // The full 70-char name should NOT appear in the chrome bar (max 60)
    let chrome = h.row(0);
    assert!(
        !chrome.contains(&long_name),
        "long title should be truncated, not shown in full"
    );
    // But the truncated version with ellipsis should appear
    assert!(chrome.contains('…'), "truncated title should have ellipsis: {chrome}");
}

#[test]
fn narrow_panel_shrinks_title_further() {
    let dir = temp_project(&[("medium_filename.rs", "fn main() {}")]);
    // Use a very narrow terminal so the center panel is small
    let mut h = TestHarness::with_size(dir.path(), 60, 20);
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);

    // The title should still be visible (at least 8 chars)
    let screen = h.screen_text();
    assert!(
        screen.contains("medium") || screen.contains('…'),
        "title should be visible even in narrow panel: {screen}"
    );
}
