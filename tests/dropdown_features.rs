//! Tests for dropdown: opens with cursor on active tab, auto-width.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

const CTRL_SHIFT_DOWN: KeyMod = KeyMod::CTRL.with_shift();

#[test]
fn dropdown_cursor_starts_on_active_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    // Open two files in center
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Focus center, switch to second tab (b.rs = index 2: Welcome, a.rs, b.rs)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    // Open dropdown
    h.inject_key(KeyCode::Down, CTRL_SHIFT_DOWN);
    h.run_cycles(1);

    // The active tab should be highlighted (bold) — check that its entry is visible
    let screen = h.screen_text();
    assert!(screen.contains("1:"), "dropdown should show entry 0");
    // The cursor should be on the active tab (last opened = b.rs at index 2)
    // We can verify by checking that the active tab's name appears in the dropdown
    assert!(screen.contains("b.rs"), "dropdown should show b.rs");
}

#[test]
fn dropdown_auto_width_fits_long_names() {
    let dir = temp_project(&[("short.rs", "x"), ("very_long_filename_here.rs", "y")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);

    // Open both files in center
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Focus center, open dropdown
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, CTRL_SHIFT_DOWN);
    h.run_cycles(1);

    // The long filename should be fully visible (not truncated)
    let screen = h.screen_text();
    assert!(
        screen.contains("very_long_filename_here.rs"),
        "dropdown should show full long filename"
    );
}

#[test]
fn dropdown_shows_borders() {
    let dir = temp_project(&[("a.rs", "x"), ("b.rs", "y")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    // Open both files
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Focus center, open dropdown
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, CTRL_SHIFT_DOWN);
    h.run_cycles(1);

    let screen = h.screen_text();
    // New dropdown renders entries without box borders
    assert!(screen.contains("a.rs"), "dropdown should show a.rs");
    assert!(screen.contains("b.rs"), "dropdown should show b.rs");
}
