mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn alt_x_opens_prompt() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('x'), KeyMod { ctrl: false, alt: true, shift: false });
    h.run_cycles(1);
    let last_row = h.row(23);
    assert!(last_row.contains(":"), "expected prompt, got: {}", last_row);
}

#[test]
fn esc_cancels_prompt() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Char('x'), KeyMod { ctrl: false, alt: true, shift: false });
    h.run_cycles(1);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    let last_row = h.row(23);
    assert!(last_row.contains("F1:Help"));
}

#[test]
fn quit_command_emits_quit() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Char('x'), KeyMod { ctrl: false, alt: true, shift: false });
    h.inject_str("quit\n");
    h.run_cycles(2);
}

#[test]
fn tab_completes_command() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Char('x'), KeyMod { ctrl: false, alt: true, shift: false });
    h.inject_str("he");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    let last_row = h.row(23);
    assert!(last_row.contains("help"), "expected completion, got: {}", last_row);
}

// --- BUG 1: M-x "open README.md" opens file ---
#[test]
fn open_command_opens_file() {
    let dir = temp_project(&[("README.md", "# Hello World\nThis is content.")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('x'), KeyMod { ctrl: false, alt: true, shift: false });
    h.inject_str("open README.md\n");
    h.run_cycles(2);
    let screen = h.screen_text();
    assert!(screen.contains("README.md"), "expected tab title, got:\n{}", screen);
    assert!(screen.contains("Hello World"), "expected content, got:\n{}", screen);
}

// --- BUG 3: Tab completion for file paths ---
#[test]
fn tab_completes_file_path() {
    let dir = temp_project(&[("README.md", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('x'), KeyMod { ctrl: false, alt: true, shift: false });
    h.inject_str("open READ");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    let last_row = h.row(23);
    assert!(last_row.contains("README.md"), "expected completed path, got: {}", last_row);
}
