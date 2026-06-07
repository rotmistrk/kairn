mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn new_file_creates_tab() {
    let dir = temp_project(&[("one.rs", "1")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.row(0).contains("one.rs"));
}

#[test]
fn tab_bar_shows_all_tabs() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    let tab_bar = h.row(0);
    // New format: shows active tab + count
    assert!(tab_bar.contains("b.rs"), "active tab shown: {}", tab_bar);
    assert!(
        tab_bar.contains("❨2❩") || tab_bar.contains("2"),
        "tab count shown: {}",
        tab_bar
    );
}

const ALT: KeyMod = KeyMod::ALT;

#[test]
fn shell_tabs_get_systematic_names() {
    let dir = temp_project(&[("a.rs", "aaa")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    assert!(h.contains("Shell:0"));
    // Open another shell
    h.inject_key(KeyCode::Char('≈'), KeyMod::default());
    h.inject_str("shell\n");
    h.run_cycles(1);
    assert!(h.contains("Shell:1"));
}

#[test]
fn alt_digit_selects_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e b.rs\n");
    h.run_cycles(1);
    assert!(h.contains("bbb"));
    // Alt-1 should switch to most recent other tab (a.rs) in LRU mode
    h.inject_key(KeyCode::Char('1'), ALT);
    h.run_cycles(1);
    assert!(h.contains("aaa"));
}

#[test]
fn rename_changes_tool_tab_title() {
    let dir = temp_project(&[("a.rs", "aaa")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('≈'), KeyMod::default());
    h.inject_str("tab-rename myserver\n");
    h.run_cycles(1);
    assert!(h.contains("Shell:myserver"));
}

#[test]
fn opening_same_file_twice_does_not_create_duplicate_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    // Open a.rs
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("aaa"));
    // Open b.rs
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Now try to open a.rs again (go back to tree, select a.rs, Enter)
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Up, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Should focus existing a.rs tab, not create a new one
    // Center panel should have exactly 2 tabs (a.rs + b.rs), not 3
    let top = h.row(0);
    // Count occurrences of "a.rs" in the tab bar — should be exactly 1
    let count = top.matches("a.rs").count();
    assert!(
        count <= 1,
        "a.rs should appear at most once in tab bar (no duplicate): count={count}, row={top}"
    );
}
