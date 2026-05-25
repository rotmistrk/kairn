//! Tests for file tree fuzzy filter.

mod helpers;

use helpers::TestHarness;
use tempfile::TempDir;
use txv_core::event::{KeyCode, KeyMod};

fn setup_tree() -> (TempDir, TestHarness) {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
    std::fs::write(dir.path().join("lib.rs"), "").unwrap();
    std::fs::write(dir.path().join("test.txt"), "hello").unwrap();
    std::fs::create_dir(dir.path().join("src")).unwrap();
    std::fs::write(dir.path().join("src/editor.rs"), "").unwrap();
    std::fs::write(dir.path().join("src/tree.rs"), "").unwrap();
    let h = TestHarness::new(dir.path());
    (dir, h)
}

#[test]
fn slash_activates_filter_shows_prompt() {
    let (_dir, mut h) = setup_tree();
    // Focus tree (F2)
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);
    // Press / to activate filter, then type a char
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("m");
    h.run_cycles(2);
    // Should show filter prompt "/m" at bottom of tree panel
    assert!(h.content_contains("/m"), "filter prompt should show /m");
}

#[test]
fn typing_filters_tree() {
    let (_dir, mut h) = setup_tree();
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);
    // Activate filter and type "rs"
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("rs");
    h.run_cycles(2);
    // .rs files should be visible, .txt should not
    assert!(h.content_contains("main.rs"));
    assert!(!h.content_contains("test.txt"), "test.txt should be filtered out");
}

#[test]
fn escape_clears_filter() {
    let (_dir, mut h) = setup_tree();
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("rs");
    h.run_cycles(2);
    assert!(!h.content_contains("test.txt"));
    // Escape clears
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    // All files visible again
    assert!(h.content_contains("test.txt"), "test.txt should reappear after Esc");
}

#[test]
fn backspace_edits_filter() {
    let (_dir, mut h) = setup_tree();
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("txt");
    h.run_cycles(2);
    // Only test.txt visible
    assert!(h.content_contains("test.txt"));
    assert!(!h.content_contains("main.rs"));
    // Backspace to "tx" — still matches test.txt
    h.inject_key(KeyCode::Backspace, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("test.txt"));
}

#[test]
fn navigation_works_during_filter() {
    let (_dir, mut h) = setup_tree();
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('/'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("rs");
    h.run_cycles(2);
    // j/k should still navigate (not be consumed by filter)
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(2);
    // Should not crash and tree should still be visible
    assert!(h.content_contains("main.rs") || h.content_contains("lib.rs"));
}
