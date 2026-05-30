mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

/// Status bar items must not disappear when navigating between panes.
/// Regression: the M-x stretch item's allocated width was fed back as min_w
/// on subsequent layout passes, creating a ratchet that progressively dropped
/// lower-priority items as the bar could no longer fit them all.
#[test]
fn status_items_persist_after_pane_navigation() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    let x = 1;\n}")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 40);
    h.run_cycles(2);

    let status_row = h.row(39);
    assert!(status_row.contains("F1:Help"), "initial: {status_row}");
    assert!(status_row.contains("M-x"), "initial: {status_row}");

    // Navigate between panes repeatedly
    for _ in 0..10 {
        h.inject_key(KeyCode::F(3), KeyMod::default());
        h.run_cycles(2);
        h.inject_key(KeyCode::F(4), KeyMod::default());
        h.run_cycles(2);
        h.inject_key(KeyCode::F(2), KeyMod::default());
        h.run_cycles(2);
    }

    let status_row = h.row(39);
    assert!(status_row.contains("F1:Help"), "after nav: {status_row}");
    assert!(status_row.contains("^Q:Quit"), "after nav: {status_row}");
    assert!(status_row.contains("M-x"), "after nav: {status_row}");
}
