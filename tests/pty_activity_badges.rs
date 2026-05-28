//! Tests for tab badges: dirty indicators and PTY activity badges.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

// --- Dirty badge tests (editor modified indicator) ---

#[test]
fn no_dirty_dot_on_clean_file() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    // Open file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    let screen = h.screen_text();
    // The dirty dot • should NOT appear next to the filename
    assert!(!screen.contains("t.txt •"), "clean file should not have dirty dot");
}

#[test]
fn dirty_dot_appears_after_edit() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    // Open file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Enter insert mode and type
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    let screen = h.screen_text();
    assert!(screen.contains(" •"), "modified file should show dirty dot •");
}

#[test]
fn dirty_dot_disappears_after_save() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    // Open file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Edit
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    // Save with :w
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("w");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    let screen = h.screen_text();
    assert!(!screen.contains("t.txt •"), "saved file should not have dirty dot");
}

// --- PTY badge tests (activity indicators) ---
// In test mode, terminals are FallbackTerminal (not real PTY), so badge sync
// correctly skips them. We test that no spurious badges appear.

#[test]
fn no_pty_badge_on_fallback_terminal() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Even with pty_last_output set, FallbackTerminal won't get badges
    // because downcast to PtyTerminal fails
    h.state.record_pty_output(0, std::time::Instant::now());
    h.run_cycles(2);

    let screen = h.screen_text();
    let has_spinner = screen.contains('◐') || screen.contains('◑') || screen.contains('◒') || screen.contains('◓');
    assert!(!has_spinner, "FallbackTerminal should not get spinner badge");
}

#[test]
fn no_idle_badge_without_output_history() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    let screen = h.screen_text();
    assert!(!screen.contains('○'), "no idle badge without output history");
}

#[test]
fn title_not_mangled_with_asterisk() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    // Open file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Edit
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    let screen = h.screen_text();
    // Title should NOT have *t.txt (old behavior) — uses • badge instead
    assert!(
        !screen.contains("*t.txt"),
        "title should not be mangled with * prefix, got badge • instead"
    );
}
