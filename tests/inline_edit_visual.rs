//! Visual correctness tests for StructuredView and CsvView inline editing.
//! Verifies buffer cells show correct styles during editing.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};
use txv_core::palette::{palette, StyleId};

fn open_struct(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

fn open_csv(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

/// Find a row containing the given text in the buffer.
fn find_row_with(h: &TestHarness, text: &str) -> Option<u16> {
    let buf = h.backend.buffer()?;
    for y in 0..buf.height() {
        let mut row = String::new();
        for x in 0..buf.width() {
            row.push(buf.cell(x, y).ch());
        }
        if row.contains(text) {
            return Some(y);
        }
    }
    None
}

/// Count cells with EditSelection bg on a given row.
fn count_selection_cells(h: &TestHarness, y: u16) -> usize {
    let buf = h.backend.buffer().unwrap();
    let sel_bg = palette().style(StyleId::EditSelection).bg();
    (0..buf.width())
        .filter(|&x| buf.cell(x, y).style().bg() == sel_bg)
        .count()
}

/// Check that a row has CursorFocused bg somewhere.
fn has_cursor_bg(h: &TestHarness, y: u16) -> bool {
    let buf = h.backend.buffer().unwrap();
    let cursor_bg = palette().style(StyleId::CursorFocused).bg();
    (0..buf.width()).any(|x| buf.cell(x, y).style().bg() == cursor_bg)
}

// --- StructuredView tests ---

#[test]
fn struct_edit_shows_selection_on_value() {
    let json = r#"{"name":"hello"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_struct(&mut h);
    // Navigate to "name" node
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value column
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Enter to edit — selects all
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    let y = find_row_with(&h, "hello").expect("row with 'hello'");
    let sel_count = count_selection_cells(&h, y);
    // "hello" is 5 chars — should have at least 5 selected cells
    assert!(
        sel_count >= 5,
        "editing value should show selection, got {sel_count} cells"
    );
}

#[test]
fn struct_edit_typing_replaces_cleanly() {
    let json = r#"{"x":"oldval"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_struct(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    // Type "hi" — replaces "oldval"
    h.inject_str("hi");
    h.run_cycles(2);

    assert!(h.content_contains("hi"), "should show 'hi'");
    assert!(!h.content_contains("oldval"), "old text should be gone");
}

#[test]
fn struct_edit_key_shows_selection() {
    let json = r#"{"mykey":"val"}"#;
    let dir = temp_project(&[("test.json", json)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_struct(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Key column is default — Enter to edit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    let y = find_row_with(&h, "mykey").expect("row with 'mykey'");
    let sel_count = count_selection_cells(&h, y);
    assert!(
        sel_count >= 5,
        "editing key should show selection, got {sel_count} cells"
    );
}

// --- CsvView tests ---

/// Find the first data row in CSV view (row after header).
fn csv_data_row(h: &TestHarness) -> Option<u16> {
    let buf = h.backend.buffer()?;
    // Look for a row containing "30" (the age value)
    for y in 0..buf.height() {
        let mut row = String::new();
        for x in 0..buf.width() {
            row.push(buf.cell(x, y).ch());
        }
        if row.contains("30") {
            return Some(y);
        }
    }
    None
}

#[test]
fn csv_edit_shows_selection() {
    let csv = "name,age\nalice,30\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);
    // Enter to edit "alice"
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    let y = csv_data_row(&h).expect("csv data row");
    let sel_count = count_selection_cells(&h, y);
    assert!(
        sel_count >= 4,
        "editing csv cell should show selection, got {sel_count} cells"
    );
}

#[test]
fn csv_edit_typing_replaces_cleanly() {
    let csv = "name,age\nalice,30\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    h.inject_str("bob");
    h.run_cycles(2);

    assert!(h.content_contains("bob"), "should show 'bob'");
}

#[test]
fn csv_edit_has_cursor_bg() {
    let csv = "name,age\nalice,30\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    // Deselect by pressing End
    h.inject_key(KeyCode::End, KeyMod::default());
    h.run_cycles(1);

    let y = csv_data_row(&h).expect("csv data row");
    assert!(has_cursor_bg(&h, y), "editing row should have CursorFocused background");
}
