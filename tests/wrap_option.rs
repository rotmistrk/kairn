// === Feature 2: :set wrap / :set nowrap ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn wrap_true_wraps_long_line() {
    // Create a line longer than the editor width
    let long_line = "A".repeat(200);
    let dir = temp_project(&[("t.txt", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Enable wrap (should be default)
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set wrap\n");
    h.run_cycles(1);
    // With wrap, the long line should appear on multiple rows
    // Row 1 and row 2 should both contain 'A' characters
    let row1 = h.row(1);
    let row2 = h.row(2);
    let a_count_row1 = row1.chars().filter(|c| *c == 'A').count();
    let a_count_row2 = row2.chars().filter(|c| *c == 'A').count();
    assert!(a_count_row1 > 10, "row1 should have A chars: {row1:?}");
    assert!(a_count_row2 > 10, "row2 should have A chars when wrap=true: {row2:?}");
}

#[test]
fn nowrap_truncates_long_line() {
    let long_line = "A".repeat(200);
    let dir = temp_project(&[("t.txt", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Disable wrap
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set nowrap\n");
    h.run_cycles(1);
    // With nowrap, only one row should have A chars (truncated)
    let row1 = h.row(1);
    let row2 = h.row(2);
    let a_count_row1 = row1.chars().filter(|c| *c == 'A').count();
    let a_count_row2 = row2.chars().filter(|c| *c == 'A').count();
    assert!(a_count_row1 > 10, "row1 should have A chars: {row1:?}");
    assert_eq!(a_count_row2, 0, "row2 should be empty when nowrap: {row2:?}");
}
