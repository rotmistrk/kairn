mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn f2_focuses_tree_slot() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
}

#[test]
fn f3_focuses_center_slot() {
    let dir = temp_project(&[("x.rs", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("content"));
}

#[test]
fn f4_focuses_right_slot() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.inject_key(KeyCode::F(4), KeyMod::default());
    h.run_cycles(1);
    assert!(
        h.contains("[Shell]") || h.contains("Shell"),
        "right slot should be focused showing Shell: {}",
        h.screen_text()
    );
}

#[test]
fn f5_toggles_zoom() {
    let dir = temp_project(&[("a.rs", "zoomed content")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("zoomed content"));
    h.inject_key(KeyCode::F(5), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
}
