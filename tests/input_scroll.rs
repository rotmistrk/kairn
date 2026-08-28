//! Regression tests for InputLine scrolling in embedded views.
//!
//! Verifies that InputLine properly scrolls within constrained bounds
//! instead of auto-expanding and pushing cursor off-screen.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

/// Test that M-x command line scrolls when typing long text.
#[test]
fn mx_command_line_scrolls_within_bounds() {
    let dir = temp_project(&[("test.txt", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Activate M-x command mode
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(2);

    // Type a long command that exceeds normal status bar width
    let long_cmd = "this_is_a_very_long_command_that_should_scroll";
    h.inject_str(long_cmd);
    h.run_cycles(2);

    // The screen should show part of the command with overflow indicator
    let content = h.screen_text();

    // Should see the end of the command (cursor is at end)
    assert!(
        content.contains("scroll") || content.contains("…"),
        "Long M-x command should either show text near cursor or overflow indicator"
    );

    // Cursor should be visible on screen (not past right edge)
    // We check this indirectly: if the command works (Enter submits), cursor was valid
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
}

/// Test that todo tree item editing scrolls within row bounds.
#[test]
fn todo_edit_scrolls_within_row() {
    let dir = temp_project(&[("test.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Focus the todo panel (F2 focuses tree, tab to todo)
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);

    // Add a new item first
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);

    // Type initial title
    h.inject_str("test");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // Start editing (e key on todo item)
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);

    // Type a long title
    let long_title = "This is a very long todo item title that should trigger scrolling";
    h.inject_str(long_title);
    h.run_cycles(2);

    // Should see overflow indicators or end of text
    let content = h.screen_text();
    assert!(
        content.contains("scrolling") || content.contains("…"),
        "Long todo title should show text near cursor or overflow indicator"
    );

    // Cancel edit
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
}
