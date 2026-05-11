mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::Event;

#[test]
fn resize_recomputes_layout() {
    let dir = temp_project(&[("a.rs", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.backend.inject(Event::Resize(60, 20));
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
}

#[test]
fn small_terminal_still_renders() {
    let dir = temp_project(&[("a.rs", "hi")]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(1);
    assert!(!h.screen_text().is_empty());
}

#[test]
fn terminal_panel_gets_full_width_in_tall() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    // In tall mode, the shell panel (bottom) chrome should span full width
    let shell_row = (0..23).find(|&y| h.row(y).contains("Shell"));
    assert!(shell_row.is_some(), "Shell chrome should be visible");
    let row = h.row(shell_row.unwrap());
    let dash_count = row.chars().filter(|&c| c == '─').count();
    assert!(
        dash_count > 40,
        "bottom chrome should span most of the width, got {dash_count} dashes"
    );
}

#[test]
fn resize_wide_to_tall_moves_shell_to_bottom() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Wide: Shell in top chrome
    let top = h.row(0);
    assert!(top.contains("Shell"), "wide should show Shell in top");

    // Resize to tall
    h.backend.set_size(80, 24);
    h.run_cycles(1);

    // Tall: Shell NOT in top chrome
    let top = h.row(0);
    assert!(!top.contains("Shell"), "tall should not show Shell in top: {top:?}");
}
