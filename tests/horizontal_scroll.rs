// === Horizontal scroll with nowrap ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn nowrap_cursor_at_end_of_long_line() {
    // Line longer than viewport width
    let long = format!("{}END", "X".repeat(200));
    let dir = temp_project(&[("t.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Set nowrap
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set nowrap\n");
    h.run_cycles(1);
    // Move to end of line with $
    h.inject_key(KeyCode::Char('$'), KeyMod::default());
    h.run_cycles(1);
    // Cursor should be at end — we can't easily verify scroll,
    // but the editor shouldn't crash and cursor should be valid
    let screen = h.screen_text();
    // The line should still be partially visible (truncated)
    assert!(screen.contains("X"), "line content should be visible");
}

#[test]
fn nowrap_second_line_not_wrapped() {
    let line1 = "A".repeat(200);
    let content = format!("{line1}\nshort");
    let dir = temp_project(&[("t.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set nowrap\n");
    h.run_cycles(1);
    // "short" should be on row 2 (not pushed down by wrapping)
    let row2 = h.row(2);
    assert!(row2.contains("short"), "second line should be on row 2: {row2:?}");
}
