mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn enter_opens_file_in_center_slot() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.row(0).contains("main.rs"));
}

#[test]
fn open_same_file_twice_focuses_existing() {
    let dir = temp_project(&[("only.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    let count = screen.matches("only.rs").count();
    assert!(count <= 2, "file opened twice: found {} occurrences", count);
}

#[test]
fn center_slot_shows_file_content() {
    let dir = temp_project(&[("hello.rs", "fn greet() { println!(\"hi\"); }")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("fn greet()"));
}

#[test]
fn multiple_files_create_multiple_tabs() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    let tab_bar = h.row(0);
    // New format: shows active tab + count "b.rs (2)"
    assert!(tab_bar.contains("b.rs"), "active tab should be shown: {}", tab_bar);
    assert!(
        tab_bar.contains("❨2❩") || tab_bar.contains("2"),
        "tab count should be shown: {}",
        tab_bar
    );
}
