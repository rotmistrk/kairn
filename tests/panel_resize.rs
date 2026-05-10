//! Tests for panel resize (≠–) and other final features.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn resize_grow_shrink_changes_layout() {
    let dir = temp_project(&[("a.rs", "aaa")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);
    // Focus left (tree) slot
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(1);
    // Get initial tree width by checking where content starts
    let row_before = h.row(1);
    // Grow: ≠
    h.inject_key(KeyCode::Char('≠'), KeyMod::default());
    h.run_cycles(1);
    let row_after = h.row(1);
    // After grow, the tree area should be wider (more chars before divider)
    assert_ne!(row_before, row_after, "resize should change layout");
}
