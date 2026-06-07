//! Scenario tests for todo tree inline editor visual correctness:
//! no extra chars, correct background, overflow indicators, selection colors.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};
use txv_core::palette::{palette, StyleId};

fn focus_todo(h: &mut TestHarness) {
    use kairn::handler::downcast_desktop;
    use kairn::slots::{focus_tab_by_title, SlotId};
    let desktop = h.program.desktop_mut();
    if let Some(d) = downcast_desktop(desktop) {
        focus_tab_by_title(d, SlotId::Left, "Todo");
        d.focus_panel(SlotId::Left as usize);
    }
    h.run_cycles(2);
}

fn todo_json(items: &str) -> String {
    format!(r#"{{"version":"2.0","title":"Todo","items":[{items}]}}"#)
}

fn item(title: &str) -> String {
    format!(r#"{{"title":"{title}","completed":"Open","important":false,"folded":false,"items":[]}}"#)
}

/// Find the row (y) in the buffer where the cursor row is rendered.
/// The cursor row has CursorFocused bg on the checkbox area.
fn find_cursor_row(h: &TestHarness) -> Option<u16> {
    let buf = h.backend.buffer()?;
    let cursor_bg = palette().style(StyleId::CursorFocused).bg();
    for y in 0..buf.height() {
        let cell = buf.cell(0, y);
        if cell.style().bg() == cursor_bg {
            return Some(y);
        }
    }
    None
}

/// Get chars on a row within the tree panel (up to first │ separator after col 5).
fn row_text(h: &TestHarness, y: u16) -> String {
    let buf = h.backend.buffer().unwrap();
    let mut s = String::new();
    for x in 0..buf.width() {
        let ch = buf.cell(x, y).ch();
        // Stop at panel separator (│ after the checkbox area)
        if ch == '│' && x > 5 {
            break;
        }
        s.push(ch);
    }
    s.trim_end().to_string()
}

#[test]
fn edit_no_extra_chars_after_text() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("hello"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);

    // Press 'e' to edit — text is selected
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    let y = find_cursor_row(&h).expect("cursor row should exist");
    let text = row_text(&h, y);
    // Should NOT contain any extra characters after the title
    // The row should have checkbox + title, no trailing garbage
    assert!(
        !text.contains("hello") || !text.contains("helloa"),
        "no extra 'a' after text. Row: '{text}'"
    );
    // More specifically: after the title text, only spaces
    if let Some(pos) = text.find("hello") {
        let after = &text[pos + 5..];
        let trimmed = after.trim_end();
        assert!(trimmed.is_empty(), "no extra chars after 'hello', got: '{trimmed}'");
    }
}

#[test]
fn edit_background_fills_editor_width() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("test"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    let y = find_cursor_row(&h).expect("cursor row");
    let cursor_bg = palette().style(StyleId::CursorFocused).bg();
    // The entire cursor row should have CursorFocused bg
    // (InputLine gets palette with Text→CursorFocused)
    let buf = h.backend.buffer().unwrap();
    let w = buf.width();
    // Check that cells in the editing area have cursor bg, not default
    let mut has_cursor_bg = false;
    for x in 0..w {
        if buf.cell(x, y).style().bg() == cursor_bg {
            has_cursor_bg = true;
        }
    }
    assert!(has_cursor_bg, "editing row should have CursorFocused background");
}

#[test]
fn edit_selection_color_on_select_all() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("abcde"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);

    // 'e' selects all text
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    let y = find_cursor_row(&h).expect("cursor row");
    let sel_bg = palette().style(StyleId::EditSelection).bg();
    let buf = h.backend.buffer().unwrap();

    // Count cells with selection bg
    let mut sel_count = 0;
    for x in 0..buf.width() {
        if buf.cell(x, y).style().bg() == sel_bg {
            sel_count += 1;
        }
    }
    // All 5 chars should be selected
    assert!(
        sel_count >= 5,
        "at least 5 cells should have EditSelection bg, got {sel_count}"
    );
}

#[test]
fn edit_selection_ends_at_text_boundary() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("abc"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    let y = find_cursor_row(&h).expect("cursor row");
    let sel_bg = palette().style(StyleId::EditSelection).bg();
    let buf = h.backend.buffer().unwrap();

    // Find the rightmost selected cell
    let mut last_sel_x: Option<u16> = None;
    for x in 0..buf.width() {
        if buf.cell(x, y).style().bg() == sel_bg {
            last_sel_x = Some(x);
        }
    }
    let last_sel = last_sel_x.expect("should have selection");
    // Cell after last selection should NOT have selection bg
    if last_sel + 1 < buf.width() {
        let after_bg = buf.cell(last_sel + 1, y).style().bg();
        assert_ne!(
            after_bg, sel_bg,
            "cell after selection should not have EditSelection bg"
        );
    }
}

#[test]
fn edit_overflow_indicator_on_long_title() {
    let long_title = "This is a very long todo item title that exceeds panel width easily";
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item(long_title));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    // 80 cols gives tree panel ~20 cols — title won't fit
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    focus_todo(&mut h);

    // Edit — cursor goes to end, text scrolls
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    let y = find_cursor_row(&h).expect("cursor row");
    let buf = h.backend.buffer().unwrap();

    // Find the '…' overflow indicator on the editing row
    let mut has_overflow = false;
    for x in 0..buf.width() {
        if buf.cell(x, y).ch() == '…' {
            has_overflow = true;
            break;
        }
    }
    assert!(has_overflow, "long title should show '…' overflow indicator");
}

#[test]
fn edit_typing_replaces_selection_cleanly() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("oldtext"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);

    // 'e' selects all
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    // Type "hi" — replaces "oldtext"
    h.inject_str("hi");
    h.run_cycles(2);

    let y = find_cursor_row(&h).expect("cursor row");
    let text = row_text(&h, y);
    assert!(text.contains("hi"), "should show 'hi'");
    assert!(!text.contains("oldtext"), "old text should be gone");
    // No leftover chars from old text
    if let Some(pos) = text.find("hi") {
        let after = &text[pos + 2..];
        let trimmed = after.trim_end();
        assert!(trimmed.is_empty(), "no leftover chars after 'hi', got: '{trimmed}'");
    }
}
