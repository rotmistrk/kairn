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

const ALT: KeyMod = KeyMod {
    ctrl: false,
    alt: true,
    shift: false,
};

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
    // Alt-0 should switch to first tab (a.rs)
    h.inject_key(KeyCode::Char('0'), ALT);
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
