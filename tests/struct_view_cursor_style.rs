//! Test: struct view cursor row — non-focused columns use normal style, not dim.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn cursor_row_non_focused_columns_are_normal_not_dim() {
    let json = r#"{"name":"hello","count":42}"#;
    let dir = temp_project(&[("data.json", json)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Focus center (struct view)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "name" row
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);

    let buf = h.backend.buffer().expect("buffer should exist");
    let pal = txv_core::palette::palette();
    let dim_style = pal.style(txv_core::palette::StyleId::Dim);
    let cursor_bg = pal.style(txv_core::palette::StyleId::CursorFocused).bg();
    let row_bg = pal.style(txv_core::palette::StyleId::CursorUnfocused).bg();

    // Find the cursor row by looking for a cell with cursor_focused bg
    let mut cursor_y: Option<u16> = None;
    for y in 1..23u16 {
        for x in 0..80u16 {
            let cell = buf.cell(x, y);
            if cell.style().bg() == cursor_bg && cell.ch() != ' ' {
                cursor_y = Some(y);
                break;
            }
        }
        if cursor_y.is_some() {
            break;
        }
    }
    let y = cursor_y.expect("should find cursor row with cursor_focused bg");

    // Check that non-cursor text cells on this row have row highlight bg (dark gray),
    // NOT dim fg. Separators (│) intentionally use dim style; we only check data cells.
    let mut found_non_cursor_text = false;
    for x in 0..80u16 {
        let cell = buf.cell(x, y);
        if cell.ch() != ' ' && cell.ch() != '│' && cell.style().bg() != cursor_bg {
            found_non_cursor_text = true;
            assert_eq!(
                cell.style().bg(),
                row_bg,
                "non-focused column text '{}' at ({x},{y}) should have row highlight bg",
                cell.ch()
            );
            assert_ne!(
                cell.style(),
                dim_style,
                "non-focused column text '{}' at ({x},{y}) should not be dim",
                cell.ch()
            );
        }
    }
    assert!(found_non_cursor_text, "should find non-cursor text on cursor row");
}
