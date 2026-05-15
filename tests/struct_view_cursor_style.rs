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

    let surface = h.backend.surface().expect("surface should exist");
    let pal = txv_core::palette::palette();
    let dim_style = pal.base.dim.to_style();
    let cursor_style = pal.interactive.cursor_focused.to_style();

    // Find the cursor row by looking for a cell with cursor_focused style
    let mut cursor_y: Option<u16> = None;
    for y in 1..23u16 {
        for x in 0..80u16 {
            let cell = surface.cell(x, y);
            if cell.style == cursor_style && cell.ch != ' ' {
                cursor_y = Some(y);
                break;
            }
        }
        if cursor_y.is_some() {
            break;
        }
    }
    let y = cursor_y.expect("should find cursor row with cursor_focused style");

    // Check that non-cursor, non-separator text cells on this row are NOT dim.
    // Separators (│) intentionally use dim style; we only check data cells.
    let mut found_non_cursor_text = false;
    for x in 0..80u16 {
        let cell = surface.cell(x, y);
        if cell.ch != ' ' && cell.ch != '│' && cell.style != cursor_style {
            found_non_cursor_text = true;
            assert_ne!(
                cell.style, dim_style,
                "non-focused column text '{}' at ({x},{y}) should be normal, not dim",
                cell.ch
            );
        }
    }
    assert!(found_non_cursor_text, "should find non-cursor text on cursor row");
}
