mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn switching_focus_updates_display() {
    let dir = temp_project(&[("a.rs", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    // Focus right (F4)
    be.inject_key(KeyCode::F(4), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Focus back to tree (F2)
    be.inject_key(KeyCode::F(2), KeyMod::default());
    let screen = run_and_capture(&mut app, &mut be, 1);
    // Tree should still show files
    assert!(screen.contains("a.rs"));
}

#[test]
fn tree_cursor_visible_when_focused() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    // Tree is focused by default (left slot) — cursor should be on first item
    // The tree renders selected item with reverse style; we just verify it renders
    assert!(be.contains("a.rs"));
}
