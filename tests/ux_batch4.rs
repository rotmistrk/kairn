//! Tests for Batch 4 UX improvements:
//! 1. Enter on tree focuses center
//! 2. Welcome view when center is empty
//! 3. Layout auto-detect (Wide vs Tall)
//! 4. Ctrl-Shift-Up/Down tab cycling (LRU)

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

// ─── Feature 1: Enter on tree focuses center ───────────────────────────────

#[test]
fn enter_on_tree_file_focuses_center() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    // Tree is focused by default. Press Right to open file and focus center.
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    // After opening, focus should be on center (file content visible with
    // focused tab style). The tree cursor should NOT be highlighted.
    // Verify by checking that the center tab has focused style (cyan/blue).
    // We verify indirectly: the focused slot's active tab renders with
    // the focused_tab_style (fg=14, bg=4). We check via focused_slot accessor.
    assert!(h.contains("fn main()"), "file content should be visible");
    // The focused slot should be Center, not Left.
    // We test this by pressing a key that only the editor handles (like 'j')
    // and verifying it moves the cursor (not the tree).
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // If center is focused, 'j' is consumed by editor. If tree is focused,
    // 'j' would do nothing useful. We verify center has focus by checking
    // that F2 (focus tree) changes something — i.e., we're NOT already on tree.
    // Better: use the public focused_slot() accessor.
    let desktop = h.program.desktop_mut();
    let sd = kairn::handler::downcast_desktop(desktop).unwrap();
    assert_eq!(sd.focused_slot(), kairn::layout_group::SlotId::Center);
}

// ─── Feature 2: Welcome view when center is empty ──────────────────────────

#[test]
fn welcome_view_shown_on_startup() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // Center should have a Welcome tab on startup
    assert!(h.contains("Welcome") || h.row(0).contains("Welcome"));
}

#[test]
fn welcome_view_closed_after_opening_file() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // Precondition: Welcome must exist first
    assert!(
        h.contains("Welcome") || h.row(0).contains("Welcome"),
        "precondition: Welcome tab must exist on startup"
    );
    // Open a file from tree
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Welcome tab should be gone
    let tab_bar = h.row(0);
    assert!(
        !tab_bar.contains("Welcome"),
        "Welcome tab should be closed after opening file"
    );
}

// ─── Feature 3: Layout auto-detect (Wide vs Tall) ──────────────────────────

#[test]
fn wide_layout_three_columns_at_200_width() {
    let dir = temp_project(&[("a.rs", "wide")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 24);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // In wide layout (>=200), all 3 slots are side by side on the same row.
    // The right slot (Shell) should be on the same row as center content.
    // Check that row 1 has both file content and shell indicator on same line.
    let desktop = h.program.desktop_mut();
    let sd = kairn::handler::downcast_desktop(desktop).unwrap();
    let rects = sd.layout_rects();
    let left = rects[kairn::layout_group::SlotId::Left as usize];
    let center = rects[kairn::layout_group::SlotId::Center as usize];
    let right = rects[kairn::layout_group::SlotId::Right as usize];
    let bottom = rects[kairn::layout_group::SlotId::Bottom as usize];
    // All three top slots should be on the same y, bottom should be empty
    assert_eq!(left.y, center.y);
    assert_eq!(center.y, right.y);
    assert_eq!(bottom.h, 0, "wide layout should not have bottom slot");
    assert!(right.w > 0, "right slot should be visible in wide layout");
}

#[test]
fn tall_layout_tools_below_at_100_width() {
    let dir = temp_project(&[("a.rs", "tall")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 24);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // In tall layout (<200), the right slot content moves to bottom.
    let desktop = h.program.desktop_mut();
    let sd = kairn::handler::downcast_desktop(desktop).unwrap();
    let rects = sd.layout_rects();
    let right = rects[kairn::layout_group::SlotId::Right as usize];
    let bottom = rects[kairn::layout_group::SlotId::Bottom as usize];
    // In tall layout, right slot should be empty (w=0) and bottom should have content
    assert_eq!(right.w, 0, "right slot should be hidden in tall layout");
    assert!(bottom.h > 0, "bottom slot should be visible in tall layout");
}

// ─── Feature 4: Ctrl-Shift-Up/Down tab cycling (LRU) ───────────────────────

#[test]
fn ctrl_shift_down_cycles_to_previous_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    // Open 3 files: a.rs, b.rs, c.rs (c.rs is active last)
    h.inject_key(KeyCode::Right, KeyMod::default()); // open a.rs + focus
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default()); // back to tree
    h.inject_key(KeyCode::Down, KeyMod::default()); // move to b.rs
    h.inject_key(KeyCode::Right, KeyMod::default()); // open b.rs + focus
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default()); // back to tree
    h.inject_key(KeyCode::Down, KeyMod::default()); // move to c.rs
    h.inject_key(KeyCode::Right, KeyMod::default()); // open c.rs + focus
    h.run_cycles(1);
    // Now c.rs is active. Ctrl-Shift-Down opens dropdown, press '1' for b.rs
    h.inject_key(
        KeyCode::Down,
        KeyMod {
            ctrl: true,
            alt: false,
            shift: true,
        },
    );
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('1'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("bbb"), "should switch to b.rs via dropdown");
}

#[test]
fn ctrl_shift_down_twice_cycles_further_back() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    // Open dropdown and press '0' to select a.rs directly
    h.inject_key(
        KeyCode::Down,
        KeyMod {
            ctrl: true,
            alt: false,
            shift: true,
        },
    );
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('0'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("aaa"), "should switch to a.rs via dropdown '0'");
}
