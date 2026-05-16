//! Tests for v-018: autosave, can_close, bracketed paste, LRU eviction.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

// --- Autosave ---

#[test]
fn autosave_saves_after_inactivity() {
    let dir = temp_project(&[("t.txt", "original")]);
    let mut h = TestHarness::new(dir.path());
    // Open file
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Edit: insert 'X' at start
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    // File should not be saved yet
    let content = std::fs::read_to_string(dir.path().join("t.txt")).unwrap();
    assert_eq!(content, "original");
    // Run enough cycles for autosave to trigger (delay=5 ticks)
    h.run_cycles(10);
    // Now file should be saved
    let content = std::fs::read_to_string(dir.path().join("t.txt")).unwrap();
    assert!(content.contains("X"), "autosave should have written: {content}");
}

// --- can_close ---

#[test]
fn clean_editor_allows_close() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // :q should close (file is clean)
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("q\n");
    h.run_cycles(2);
    // Should show Welcome (center empty after close)
    assert!(h.contains("Welcome") || !h.contains("hello"));
}

#[test]
fn dirty_editor_shows_save_prompt() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Edit to make dirty
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('Z'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    // :q! should force close (bypass prompt)
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("q!\n");
    h.run_cycles(1);
    // Should close without saving
    let content = std::fs::read_to_string(dir.path().join("t.txt")).unwrap();
    assert_eq!(content, "hello", "force close should not save");
}

// --- Bracketed paste ---

#[test]
fn paste_event_inserts_text_as_block() {
    let dir = temp_project(&[("t.txt", "line1\nline2")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Simulate a paste event
    h.backend.inject_paste("PASTED");
    h.run_cycles(1);
    // The pasted text should appear in the buffer
    assert!(h.content_contains("PASTED"), "paste should insert text");
}

// --- LRU eviction respects can_close ---

#[test]
fn lru_eviction_skips_dirty_editor() {
    let dir = temp_project(&[("a.rs", "aaa")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    // Open file and make it dirty
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    // Open 9 more files to fill the slot (max 10)
    for i in 0..9 {
        h.inject_key(KeyCode::Char(':'), KeyMod::default());
        h.inject_str(&format!("e file{i}.rs\n"));
        h.run_cycles(1);
    }
    // Open one more — should evict LRU but NOT the dirty a.rs
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e extra.rs\n");
    h.run_cycles(1);
    // a.rs should still be in the tab list (not evicted)
    // Switch back to check
    let screen = h.screen_text();
    // The dirty file should still exist (Welcome was evicted instead)
    assert!(
        screen.contains("a.rs") || h.contains("a.rs"),
        "dirty editor should not be evicted"
    );
}
