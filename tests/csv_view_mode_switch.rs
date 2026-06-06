//! Scenario tests for CSV view mode switching (M-x text / M-x struct).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn alt(ch: char) -> (KeyCode, KeyMod) {
    (
        KeyCode::Char(ch),
        KeyMod {
            alt: true,
            ..KeyMod::default()
        },
    )
}

/// Open the only file in temp project as CSV table view.
fn open_csv(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

/// Execute a command via M-x.
fn mx_command(h: &mut TestHarness, cmd: &str) {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    h.inject_str(cmd);
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
}

#[test]
fn csv_mx_text_switches_to_plain_editor() {
    let csv = "name,age\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    // Verify we're in CSV table view — "alice" and "30" in separate cells
    assert!(h.content_contains("alice"), "should see alice in CSV view");

    // M-x text should switch to plain text editor
    mx_command(&mut h, "text");

    // Plain editor shows raw CSV text with commas (not table-formatted)
    assert!(
        h.content_contains("alice,30"),
        "plain editor should show raw CSV with comma: {}",
        h.screen_text()
    );
}

#[test]
fn csv_mx_struct_stays_structured() {
    let csv = "name,age\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    // M-x struct should re-open as structured (CSV table)
    mx_command(&mut h, "struct");

    // Should still be in table view
    assert!(
        h.content_contains("│"),
        "struct command should keep CSV in table view: {}",
        h.screen_text()
    );
}

#[test]
fn text_mx_struct_switches_to_table() {
    let csv = "name,age\nalice,30\nbob,25\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    // Open as plain text first
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // It auto-opens as CSV. Switch to text first.
    mx_command(&mut h, "text");
    assert!(h.content_contains("alice,30"), "should be in text mode first");

    // Now switch back to struct
    mx_command(&mut h, "struct");
    assert!(
        h.content_contains("│"),
        "struct should switch CSV back to table view: {}",
        h.screen_text()
    );
}

#[test]
fn json_mx_text_switches_to_plain() {
    let json = r#"{"name":"alice","age":30}"#;
    let dir = temp_project(&[("data.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Should be in structured view (shows key/value columns)
    assert!(h.content_contains("name"), "struct view should show 'name'");

    // M-x text
    mx_command(&mut h, "text");

    // Plain text editor shows raw JSON
    assert!(
        h.content_contains(r#""name""#),
        "plain editor should show raw JSON with quotes: {}",
        h.screen_text()
    );
}

#[test]
fn json_mx_struct_from_text() {
    let json = r#"{"name":"alice","age":30}"#;
    let dir = temp_project(&[("data.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Switch to text first
    mx_command(&mut h, "text");
    // Then back to struct
    mx_command(&mut h, "struct");

    // Should be back in structured view
    assert!(
        h.content_contains("name") && h.content_contains("alice"),
        "should be back in structured view: {}",
        h.screen_text()
    );
}

#[test]
fn csv_numeric_right_aligned() {
    let csv = "item,count\napple,5\nbanana,123\ncherry,42\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);

    // Find the row with "apple" — the number "5" should be right-aligned
    // In right-alignment, "5" is preceded by spaces: "  5" not "5  "
    let screen = h.screen_text();
    let apple_line = screen
        .lines()
        .find(|l| l.contains("apple"))
        .expect("should find apple row");
    // After the last │ before "5", there should be leading spaces
    // The number column cell content should end with "5" (right-aligned)
    // Find the position of the separator after "apple"
    let after_apple = apple_line.find("apple").unwrap() + 5;
    let rest = &apple_line[after_apple..];
    // Find the next │...│ segment (the number cell)
    if let Some(sep1) = rest.find('│') {
        let cell_start = after_apple + sep1 + 3; // "│" is 3 bytes in UTF-8
        if cell_start < apple_line.len() {
            let next_sep = apple_line[cell_start..]
                .find('│')
                .unwrap_or(apple_line.len() - cell_start);
            let cell = &apple_line[cell_start..cell_start + next_sep];
            // Right-aligned: cell should end with the number (trim trailing spaces)
            let trimmed = cell.trim_end();
            assert!(
                trimmed.ends_with('5'),
                "numeric cell should be right-aligned (end with '5'), got: '{cell}'"
            );
            // And should have leading space (since 5 is shorter than 123)
            assert!(
                cell.starts_with(' '),
                "right-aligned '5' should have leading space, got: '{cell}'"
            );
        }
    }
}

#[test]
fn csv_sort_numeric_column() {
    let csv = "item,count\napple,5\nbanana,123\ncherry,42\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);

    // Move to count column
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    // Sort
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(2);

    let screen = h.screen_text();
    let pos_5 = screen.find("apple").unwrap_or(usize::MAX);
    let pos_42 = screen.find("cherry").unwrap_or(usize::MAX);
    let pos_123 = screen.find("banana").unwrap_or(usize::MAX);
    // Numeric sort: 5 < 42 < 123
    assert!(
        pos_5 < pos_42 && pos_42 < pos_123,
        "should be sorted numerically: apple(5) < cherry(42) < banana(123)"
    );
}

#[test]
fn csv_decimal_alignment() {
    // Values with different decimal lengths should align on the dot
    let csv = "label,value\na,1.5\nb,12.345\nc,7.0\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);

    let screen = h.screen_text();
    // Find the dot positions in each row — they should all be at the same column
    let mut dot_cols: Vec<usize> = Vec::new();
    for line in screen.lines() {
        if line.contains("1.5") || line.contains("12.345") || line.contains("7.0") {
            if let Some(dot_pos) = line.find('.') {
                dot_cols.push(dot_pos);
            }
        }
    }
    assert!(
        dot_cols.len() >= 2,
        "should find at least 2 rows with dots, got: {:?}\n{}",
        dot_cols,
        screen
    );
    // All dots should be at the same column position
    let first = dot_cols[0];
    for (i, &col) in dot_cols.iter().enumerate() {
        assert_eq!(
            col, first,
            "dot in row {i} at col {col} != expected col {first} (dots should align)"
        );
    }
}

#[test]
fn csv_scientific_notation_preserved() {
    // Scientific notation values should be preserved in original form
    let csv = "name,measurement\nalpha,1.23e+07\nbeta,4.5e-3\ngamma,100\n";
    let dir = temp_project(&[("data.csv", csv)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_csv(&mut h);

    let screen = h.screen_text();
    // Original forms must be preserved
    assert!(
        screen.contains("1.23e+07"),
        "scientific notation must be preserved: {}",
        screen
    );
    assert!(
        screen.contains("4.5e-3"),
        "scientific notation must be preserved: {}",
        screen
    );
    assert!(screen.contains("100"), "integer form must be preserved: {}", screen);
}
