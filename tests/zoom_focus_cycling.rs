//! Tests for focus cycling in zoom mode (task 006).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

const CTRL_SHIFT: KeyMod = KeyMod {
    ctrl: true,
    alt: false,
    shift: true,
};

#[test]
fn cycle_focus_in_zoom_moves_to_next_panel_zoomed() {
    let dir = temp_project(&[("a.rs", "editor content")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Focus center and zoom
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    // Tree should NOT be visible (center is zoomed)
    assert!(
        !h.content_contains("a.rs"),
        "tree should be hidden when center is zoomed"
    );

    // Cycle focus right (Ctrl-Shift-Right)
    h.inject_key(KeyCode::Right, CTRL_SHIFT);
    h.run_cycles(1);

    // Now the right panel (Shell) should be zoomed — tree still hidden
    assert!(
        !h.content_contains("a.rs"),
        "tree should still be hidden after cycling to zoomed shell"
    );
    // Shell tab should be visible in chrome
    let top = h.row(0);
    assert!(
        top.contains("Shell"),
        "zoomed chrome should show Shell after cycling right: {top:?}"
    );
}

#[test]
fn cycle_focus_left_in_zoom_goes_to_tree() {
    let dir = temp_project(&[("a.rs", "editor content")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Focus center and zoom
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);

    // Cycle focus left (Ctrl-Shift-Left)
    h.inject_key(KeyCode::Left, CTRL_SHIFT);
    h.run_cycles(1);

    // Tree panel should now be zoomed — tree content visible
    assert!(
        h.content_contains("a.rs"),
        "tree should be visible when zoomed after cycling left"
    );
}

#[test]
fn cycle_focus_without_zoom_still_works() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Focus center (not zoomed)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    // Cycle right — should move focus without zooming
    h.inject_key(KeyCode::Right, CTRL_SHIFT);
    h.run_cycles(1);

    // Tree should still be visible (no zoom)
    assert!(h.content_contains("a.rs"), "tree should remain visible when not zoomed");
}
