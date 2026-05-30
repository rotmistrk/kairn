//! Test that the sidekick (M-x completion popup) renders at the correct position.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn alt() -> KeyMod {
    KeyMod {
        ctrl: false,
        alt: true,
        shift: false,
    }
}

/// M-x with partial input shows completion popup above the status bar, not at row 0.
#[test]
fn mx_sidekick_popup_appears_above_status_bar() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x
    h.inject_key(KeyCode::Char('x'), alt());
    h.run_cycles(2);

    // Type "t" — matches multiple commands (test, theme, tab-rename, toggle-tree, etc.)
    h.inject_str("t");
    h.run_cycles(5);

    // Find which row has a completion item
    let mut popup_row = None;
    for y in 0..23u16 {
        let row = h.row(y);
        if row.contains("test") && !row.contains(".rs") {
            popup_row = Some(y);
            break;
        }
    }

    assert!(popup_row.is_some(), "completion popup with 'test' should be visible");
    let popup_row = popup_row.unwrap();
    // Popup should be in the lower half (near status bar at row 23), not at the top
    assert!(
        popup_row >= 12,
        "popup should be near bottom (row 12+), but was at row {}",
        popup_row
    );
}
