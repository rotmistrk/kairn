//! Regression tests for recent features:
//! welcome returns after last tab closed, file reopen, enter without focus,
//! mode indicator, position indicator, clock in status bar.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

// ─── TEST 1: Welcome returns after last tab closed ─────────────────────────

#[test]
fn welcome_returns_after_last_tab_closed() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    // Open file via Right arrow (focuses center)
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("hello"), "file should be open");
    // Close it with :q
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("q\n");
    // Inject a tick so CM_FILE_CLOSED (queued by desktop) gets processed
    h.backend.inject(txv_core::event::Event::Tick);
    h.run_cycles(1);
    assert!(h.contains("Welcome"), "Welcome should reappear after last tab closed");
}

// ─── TEST 2: File can be reopened after close ──────────────────────────────

#[test]
fn file_can_be_reopened_after_close() {
    let dir = temp_project(&[("a.rs", "reopen_me")]);
    let mut h = TestHarness::new(dir.path());
    // Open file
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("reopen_me"));
    // Close it
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("q\n");
    h.backend.inject(txv_core::event::Event::Tick);
    h.run_cycles(1);
    // Go back to tree and reopen
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("reopen_me"), "file should be visible again after reopen");
}

// ─── TEST 3: Enter opens file without focus change ─────────────────────────

#[test]
fn enter_opens_file_without_focus_change() {
    let dir = temp_project(&[("a.rs", "content_a"), ("b.rs", "content_b")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // Tree is focused. Press Enter to open file without moving focus.
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // File should be open in center (content visible)
    assert!(h.contains("content_a"), "file should be opened in center");
    // Focus should still be on tree — pressing 'j' should move tree cursor
    let desktop = h.program.desktop_mut();
    let sd = kairn::handler::downcast_desktop(desktop).unwrap();
    assert_eq!(
        sd.focused_panel(),
        kairn::slots::SlotId::Left as usize,
        "focus should remain on tree"
    );
}

// ─── TEST 4: Mode indicator shows insert ───────────────────────────────────

#[test]
fn mode_indicator_shows_insert() {
    let dir = temp_project(&[("a.rs", "text")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("NOR"), "should show NOR in normal mode");
    // Enter insert mode
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("INS"), "should show INS in insert mode");
    // Back to normal
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("NOR"), "should show NOR after Esc");
}

// ─── TEST 5: Position indicator updates on move ────────────────────────────

#[test]
fn position_indicator_updates_on_move() {
    let dir = temp_project(&[("a.rs", "line1\nline2\nline3")]);
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(2);
    assert!(h.contains("Ln 1"), "initial position should show Ln 1");
    // Move down
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("Ln 2"), "position should update to Ln 2 after j");
}

// ─── TEST 6: Clock appears in status bar ───────────────────────────────────

#[test]
fn clock_appears_in_status_bar() {
    let dir = temp_project(&[("a.rs", "x")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let screen = h.screen_text();
    let has_time = regex::Regex::new(r"\d\d:\d\d").unwrap().is_match(&screen);
    assert!(has_time, "status bar should contain a time pattern (HH:MM)");
}
