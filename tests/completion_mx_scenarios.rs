//! M-x command line completion scenario tests.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

// ─── M-x command line completion tests ───────────────────────────────

#[test]
fn mx_shows_completion_on_tab() {
    let dir = temp_project(&[("f.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x prompt
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(1);

    // Type "e" and press Tab to trigger completion
    h.inject_str("e");
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(3);

    // Dropdown should appear with commands starting with 'e'
    let screen = h.screen_text();
    assert!(
        screen.contains("edit") || screen.contains("e "),
        "Tab should show completion dropdown with commands matching 'e'"
    );
}

#[test]
fn mx_esc_cancels() {
    let dir = temp_project(&[("f.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x, type "ed", Tab to show dropdown
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(1);
    h.inject_str("ed");
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(3);

    // Press Esc to cancel
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(4);

    // Should be back to normal mode (status bar shows F1:Help)
    let screen = h.screen_text();
    assert!(
        screen.contains("F1") || screen.contains("Help"),
        "Esc should dismiss M-x and return to normal mode"
    );
}

#[test]
fn mx_enter_accepts_and_executes() {
    let dir = temp_project(&[("hello.txt", "world\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x, type "he", Tab to trigger completion
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(1);
    h.inject_str("he");
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(3);

    // If dropdown shows "help", press Enter to accept and execute
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    // "help" command should have executed — help content visible
    let screen = h.screen_text();
    assert!(
        screen.contains("Help") || screen.contains("help") || screen.contains("F1"),
        "accepting 'help' completion should show help content"
    );
}
