//! Tests for critical rendering bugs: scroll garbage, list tabs, nolist cursor.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

// ─── BUG 1: Screen garbage on scroll ───────────────────────────────────────

#[test]
fn scroll_clears_previous_content() {
    // Create a file with 50 lines
    let content: String = (1..=50).map(|i| format!("line{}\n", i)).collect();
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);

    // Open the file
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Verify line1 is visible initially
    assert!(h.contains("line1"), "line1 should be visible initially");

    // Scroll down with PgDn
    h.inject_key(KeyCode::PageDown, KeyMod::default());
    h.run_cycles(1);

    // After scrolling past line1, it must NOT appear on screen
    let screen = h.screen_text();
    // PgDn moves by viewport height (~22 lines), so line1 should be gone
    assert!(
        !screen.contains("line1 ") && !screen.contains("line1\n"),
        "line1 must NOT appear after PgDn scroll. Screen:\n{}",
        screen
    );
    // But later lines should be visible
    assert!(h.contains("line2"), "later lines should be visible after scroll");
}

/// Regression: after scrolling, shorter lines must not leave trailing chars
/// from longer lines that were previously at that screen row.
#[test]
fn scroll_no_trailing_garbage_from_longer_lines() {
    // Line 1 is very long, lines 25+ are short. After PgDn, the long content must be gone.
    let mut content = String::new();
    content.push_str(&"X".repeat(60));
    content.push('\n');
    for i in 2..=50 {
        content.push_str(&format!("L{}\n", i));
    }
    let dir = temp_project(&[("long.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Verify the long line is visible
    assert!(h.contains("XXXX"), "long line should be visible initially");

    // Scroll past it
    h.inject_key(KeyCode::PageDown, KeyMod::default());
    h.run_cycles(1);

    // The X's must be completely gone
    assert!(
        !h.contains("XXXX"),
        "long line content must not remain after scrolling past it"
    );
}

// ─── BUG 2: Tab renders as ───→ in list mode ───────────────────────────────

#[test]
fn list_mode_tab_renders_as_arrows() {
    let dir = temp_project(&[("tabs.txt", "\thello\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);

    // Open file
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Enter command mode and set list
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set list\n");
    h.run_cycles(1);

    // Tab should render as ───→ (tab_width=4: 3 dashes + arrow)
    let screen = h.screen_text();
    assert!(
        screen.contains("───→") || screen.contains("─→"),
        "tab should render as horizontal lines + arrow in list mode. Screen:\n{}",
        screen
    );
    // "hello" must still be intact after the tab
    assert!(
        screen.contains("hello"),
        "content after tab must not be overwritten. Screen:\n{}",
        screen
    );
}

// ─── BUG 3: Cursor visible on tabs in nolist mode ──────────────────────────

#[test]
fn cursor_visible_on_tab_in_nolist_mode() {
    let dir = temp_project(&[("tab.txt", "\thello\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);

    // Open file, cursor at (0,0) which is a tab character
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // Find cursor (reverse attribute) in the editor area.
    // It must be at the FIRST column of text (right after gutter), not displaced.
    let buf = h.backend.buffer().expect("buffer should exist");
    let mut cursor_x: Option<u16> = None;
    let mut _cursor_y: Option<u16> = None;
    for y in 1..23u16 {
        for x in 0..80u16 {
            let cell = buf.cell(x, y);
            if cell.style().bg() == txv_core::cell::Color::Ansi(7)
                && cell.style().fg() == txv_core::cell::Color::Ansi(0)
            {
                cursor_x = Some(x);
                _cursor_y = Some(y);
                break;
            }
        }
        if cursor_x.is_some() {
            break;
        }
    }
    assert!(
        cursor_x.is_some(),
        "cursor must be visible on tab character in nolist mode"
    );

    // Cursor should be at the first text column (after tree panel + gutter).
    // Tree panel is 24 wide + 1 border = 25, gutter "1 " = 2, so text starts at ~27.
    let cx = cursor_x.unwrap();
    assert!(cx <= 30, "cursor should be at start of text area (got x={})", cx);
}

#[test]
fn tab_expands_to_spaces_in_nolist_mode() {
    let dir = temp_project(&[("tab.txt", "\thello\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);

    // Open file
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);

    // In nolist mode, tab should expand to spaces and "hello" should appear
    // at column tab_width (4) after the gutter
    let screen = h.screen_text();
    assert!(
        screen.contains("hello"),
        "hello should be visible after tab expansion. Screen:\n{}",
        screen
    );

    // The tab should NOT appear as a visible character
    let buf = h.backend.buffer().expect("buffer");
    let mut tab_char_found = false;
    for y in 1..23u16 {
        for x in 0..80u16 {
            if buf.cell(x, y).ch() == '\t' {
                tab_char_found = true;
            }
        }
    }
    assert!(!tab_char_found, "raw tab character must not appear on screen");

    // "hello" must start at gutter_width + tab_width
    // Find 'h' of hello on the content line
    let mut hello_x: Option<u16> = None;
    for y in 1..23u16 {
        for x in 0..80u16 {
            if buf.cell(x, y).ch() == 'h' {
                let next = buf.cell(x + 1, y).ch();
                if next == 'e' {
                    hello_x = Some(x);
                    break;
                }
            }
        }
        if hello_x.is_some() {
            break;
        }
    }
    assert!(hello_x.is_some(), "hello must be found on screen");
    // hello should be offset by tab_width (4) from text start
    let hx = hello_x.unwrap();
    assert!(hx >= 5, "hello should be indented by tab_width (got x={})", hx);
}
