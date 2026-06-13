//! Tests that verify CsvView cursor actually moves — checks visual cursor row position.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};
use txv_core::palette::{palette, StyleId};

fn open_csv(h: &mut TestHarness) {
    // Open file from tree, focus center
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

/// Find the Y position of the cursor row by checking for CursorFocused bg.
fn find_cursor_row(h: &TestHarness) -> Option<u16> {
    let buf = h.backend.buffer()?;
    let cursor_bg = palette().style(StyleId::CursorFocused).bg();
    // Start at row 1 to skip tab bar
    for y in 1..23u16 {
        for x in 0..80u16 {
            let cell = buf.cell(x, y);
            if cell.style().bg() == cursor_bg && cell.ch() != ' ' {
                return Some(y);
            }
        }
    }
    None
}

#[test]
fn csv_cursor_moves_down_on_j() {
    let csv = "name,val\nalice,1\nbob,2\ncarol,3\n";
    let dir = temp_project(&[("t.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    let y1 = find_cursor_row(&h).expect("cursor visible initially");

    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(2);

    let y2 = find_cursor_row(&h).expect("cursor visible after j");
    assert!(y2 > y1, "cursor should move down: was {y1}, now {y2}");
}

#[test]
fn csv_cursor_moves_up_on_k() {
    let csv = "name,val\nalice,1\nbob,2\ncarol,3\n";
    let dir = temp_project(&[("t.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);

    // Move down first
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(2);
    let y_after_j = find_cursor_row(&h).expect("cursor after j");

    // Move back up
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(2);
    let y_after_k = find_cursor_row(&h).expect("cursor after k");

    assert!(
        y_after_k < y_after_j,
        "cursor should move up: was {y_after_j}, now {y_after_k}"
    );
}
