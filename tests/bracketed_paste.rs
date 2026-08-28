//! Test bracketed paste preserves newlines in editor.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

/// Test that bracketed paste (terminal paste) preserves newlines in editor.
#[test]
fn bracketed_paste_preserves_newlines() {
    let dir = temp_project(&[("test.txt", "start")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open the file
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.inject_str("edit test.txt");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    // Go to end and enter insert mode
    h.inject_key(KeyCode::Char('A'), KeyMod::default()); // Append at end of line
    h.run_cycles(1);

    // Simulate bracketed paste with multiline text
    h.backend.inject_paste("\nline_two\nline_three\n");
    h.run_cycles(2);

    // Exit insert mode
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    let screen = h.screen_text();

    // Should see all three lines
    assert!(
        screen.contains("line_two"),
        "Pasted line_two should be visible: {}",
        screen
    );
    assert!(
        screen.contains("line_three"),
        "Pasted line_three should be visible: {}",
        screen
    );
}
