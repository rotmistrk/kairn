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
    assert!(tab_bar.contains("a.rs"));
    assert!(tab_bar.contains("b.rs"));
}
