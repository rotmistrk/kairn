//! Tests for dropdown tab picker positioning in wide and tall layouts.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

const CTRL_SHIFT_DOWN: KeyMod = KeyMod::CTRL.with_shift();

/// In wide layout (>=200 cols), right slot is on the right side.
/// Dropdown should appear below the right slot's chrome, not at top.
#[test]
fn dropdown_position_wide_layout_right_slot() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::with_size(dir.path(), 300, 30);
    // Focus right slot
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    // Add a second tab so dropdown can open
    h.inject_key(KeyCode::Char('≈'), KeyMod::default());
    h.inject_str("shell\n");
    h.run_cycles(1);
    // Open dropdown
    h.inject_key(KeyCode::Down, CTRL_SHIFT_DOWN);
    h.run_cycles(1);
    // The dropdown should appear in the right slot area (rows 1-3: border + content)
    let screen = h.screen_text();
    assert!(
        screen.contains("1:"),
        "dropdown should render in right slot area. Screen:\n{screen}"
    );
    // It should NOT be at the far left (that's the tree)
    let row2 = h.row(2);
    let left_part: String = row2.chars().take(10).collect();
    assert!(
        !left_part.contains("1:"),
        "dropdown should be in right slot, not left: {row2:?}"
    );
}

/// In tall layout (<200 cols), slots stack vertically.
/// Dropdown for center slot should appear below center's chrome row.
#[test]
fn dropdown_position_tall_layout_center_slot() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    // Open two files in center
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    // Focus center, open dropdown
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.inject_key(KeyCode::Down, CTRL_SHIFT_DOWN);
    h.run_cycles(1);
    // Dropdown should appear below center's chrome (not at row 0/1 which is tree area)
    // Center slot starts after the tree in tall layout
    let screen = h.screen_text();
    assert!(screen.contains("1:"), "dropdown should be visible: {screen}");
    assert!(screen.contains("a.rs"), "dropdown should list a.rs");
    assert!(screen.contains("b.rs"), "dropdown should list b.rs");
}

/// Dropdown for right slot in tall layout should appear at right slot's position.
#[test]
fn dropdown_position_tall_layout_right_slot() {
    let dir = temp_project(&[("a.rs", "aaa")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    // Focus right slot (Shell)
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    // Need at least 2 tabs for dropdown. Open another shell.
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("shell\n");
    h.run_cycles(1);
    // Open dropdown
    h.inject_key(KeyCode::Down, CTRL_SHIFT_DOWN);
    h.run_cycles(1);
    // Row 0 is tree chrome. Dropdown should NOT be there.
    let row0 = h.row(0);
    assert!(
        !row0.contains("0:"),
        "dropdown should NOT render at row 0 for right slot. Row 0: {row0:?}"
    );
}
