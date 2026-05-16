//! Tests for hysteresis layout switching and proportional widths.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn wide_at_200_cols() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 300, 30);
    h.run_cycles(1);

    // In wide mode, Shell tab should be in the top chrome (right slot visible)
    let top = h.row(0);
    assert!(
        top.contains("Shell"),
        "wide layout should show Shell in top chrome: {top:?}"
    );
}

#[test]
fn tall_at_176_cols() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // In tall mode, Shell tab should NOT be in the top chrome (it's in bottom divider)
    let top = h.row(0);
    assert!(
        !top.contains("Shell"),
        "tall layout should not show Shell in top chrome: {top:?}"
    );
    // But Shell should be visible somewhere on screen (bottom divider)
    let screen = h.screen_text();
    assert!(screen.contains("Shell"), "Shell should be visible in tall layout");
}

#[test]
fn hysteresis_stays_in_between() {
    let dir = temp_project(&[("a.rs", "hello")]);
    // Start wide (300), then resize to 250 (between thresholds)
    let mut h = TestHarness::with_size(dir.path(), 300, 30);
    h.run_cycles(1);

    // Verify wide
    let top = h.row(0);
    assert!(top.contains("Shell"), "should start wide");

    // Resize to 250 (between 200 and 300) — should stay wide
    h.backend.set_size(250, 30);
    h.run_cycles(1);

    let top = h.row(0);
    assert!(
        top.contains("Shell"),
        "at 250 cols (between thresholds), should stay wide: {top:?}"
    );
}

#[test]
fn hysteresis_switches_to_tall_at_176() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 300, 30);
    h.run_cycles(1);

    // Resize to 200 — should switch to tall
    h.backend.set_size(200, 30);
    h.run_cycles(1);

    let top = h.row(0);
    assert!(!top.contains("Shell"), "at 200 cols, should switch to tall: {top:?}");
}

#[test]
fn hysteresis_switches_to_wide_at_200() {
    let dir = temp_project(&[("a.rs", "hello")]);
    // Start tall
    let mut h = TestHarness::with_size(dir.path(), 80, 30);
    h.run_cycles(1);

    // Resize to 300 — should switch to wide
    h.backend.set_size(300, 30);
    h.run_cycles(1);

    let top = h.row(0);
    assert!(top.contains("Shell"), "at 300 cols, should switch to wide: {top:?}");
}

#[test]
fn wide_proportional_1_2_2() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 300, 30);
    h.run_cycles(1);

    // Check that the tree (left) is roughly 1/5 of width
    let row = h.row(1);
    // Find first │ divider position
    let div_pos = row.chars().position(|c| c == '│');
    assert!(
        div_pos.is_some(),
        "should have a vertical divider in wide mode: {row:?}"
    );
    let pos = div_pos.unwrap();
    // 1/5 of 300 = 60, minus 2 dividers = 298/5 = 59
    assert!(
        (55..=65).contains(&pos),
        "left panel should be ~1/5 width (59), got divider at {pos}: {row:?}"
    );
}

#[test]
fn tall_proportional_1_2_width() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 30);
    h.run_cycles(1);

    // In tall mode, left is 1/5 of width, center is rest (no right column)
    let row = h.row(1);
    let div_pos = row.chars().position(|c| c == '│');
    assert!(div_pos.is_some(), "should have divider in tall mode: {row:?}");
    let pos = div_pos.unwrap();
    // 1/5 of 80 = 16, minus 1 divider = 78/5 = 15
    assert!(
        (12..=20).contains(&pos),
        "left panel should be ~1/5 width in tall mode, got divider at {pos}: {row:?}"
    );
}
