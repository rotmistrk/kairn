//! Scenario tests for CsvView inline editing and filter.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_csv(h: &mut TestHarness) {
    // Open the CSV file, then switch to table view via M-x "tab"
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

#[test]
fn csv_edit_cell_commit() {
    let csv = "name,age\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Cursor at row 0, col 0 (alice)
    // Press Enter to edit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Clear and type new name
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    for _ in 0..5 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("carol");
    h.run_cycles(1);
    // Commit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("carol"), "cell should show 'carol' after edit");
}

#[test]
fn csv_edit_cell_cancel() {
    let csv = "name,age\nalice,30\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Type something
    h.inject_str("xxx");
    h.run_cycles(1);
    // Cancel with Esc
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    // Original value should remain
    assert!(
        h.content_contains("alice"),
        "cell should still show 'alice' after cancel"
    );
    assert!(!h.content_contains("xxx"), "cancelled text should not appear");
}

#[test]
fn csv_filter_column() {
    let csv = "name,age\nalice,30\nbob,25\ncharlie,35\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Press 'f' to start filter on column 0
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    // Type "ali" to filter
    h.inject_str("ali");
    h.run_cycles(1);
    // Commit filter
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    // Only alice should be visible
    assert!(h.content_contains("alice"), "alice should be visible");
    assert!(!h.content_contains("bob"), "bob should be filtered out");
}

#[test]
fn csv_sort_column() {
    let csv = "name,age\ncharlie,35\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Press 's' to sort by name (col 0)
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(2);
    let content = h.screen_text();
    let alice_pos = content.find("alice").unwrap_or(usize::MAX);
    let bob_pos = content.find("bob").unwrap_or(usize::MAX);
    let charlie_pos = content.find("charlie").unwrap_or(usize::MAX);
    assert!(
        alice_pos < bob_pos && bob_pos < charlie_pos,
        "should be sorted alphabetically: alice < bob < charlie"
    );
}

#[test]
fn csv_navigate_and_edit_second_column() {
    let csv = "name,age\nalice,30\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Move right to age column
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    // Edit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Clear and type new age
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Delete, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Delete, KeyMod::default());
    h.run_cycles(1);
    h.inject_str("99");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("99"), "age should be updated to 99");
}
