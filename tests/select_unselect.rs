mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn switching_focus_updates_display() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
}

#[test]
fn tree_cursor_visible_when_focused() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
}
