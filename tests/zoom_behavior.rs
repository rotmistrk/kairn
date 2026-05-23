//! Tests for zoom behavior: bounds preservation, full-size zoomed panel, draw isolation.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn zoom_only_draws_zoomed_panel() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Verify tree is visible before zoom
    assert!(h.content_contains("a.rs"), "tree should show a.rs before zoom");

    // Focus center and zoom (F5 = zoom toggle)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    // Tree content should NOT be visible — zoomed center covers everything
    assert!(
        !h.content_contains("a.rs"),
        "tree content should not be visible when center is zoomed"
    );
}

#[test]
fn unzoom_restores_normal_layout() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Zoom then unzoom
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    // Tree should be visible again
    assert!(h.content_contains("a.rs"), "tree should be visible after unzoom");
}

#[test]
fn zoom_chrome_shows_tab_name() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Focus right (Shell) and zoom
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    let top = h.row(0);
    assert!(top.contains("Shell"), "zoomed chrome should show tab name: {top:?}");
}

#[test]
fn zoom_does_not_show_dividers() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Zoom center
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    // No vertical dividers should be visible (only chrome ─ at top)
    for y in 1..29 {
        let row = h.row(y);
        assert!(
            !row.contains('┬'),
            "no ┬ connector should appear when zoomed, row {y}: {row:?}"
        );
    }
}

#[test]
fn ctrl_shift_down_opens_dropdown_in_zoom() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    // Focus right (Shell) and add second tab via :shell command
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    h.dispatch_command(kairn::commands::CM_EXECUTE_COMMAND, Some(Box::new("shell".to_string())));
    h.run_cycles(1);

    // Zoom
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    // Open dropdown with Ctrl-Shift-Down
    h.inject_key(
        KeyCode::Down,
        KeyMod {
            ctrl: true,
            alt: false,
            shift: true,
        },
    );
    h.run_cycles(1);

    let screen = h.screen_text();
    assert!(screen.contains("1:"), "dropdown should open in zoom mode");
}
