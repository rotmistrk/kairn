mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn f2_focuses_tree_slot() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Start, then press F3 to go to center, then F2 to go back
    be.inject_key(KeyCode::F(3), KeyMod::default());
    be.inject_key(KeyCode::F(2), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Tree should show files (it's focused)
    assert!(be.contains("a.rs"));
}

#[test]
fn f3_focuses_center_slot() {
    let dir = temp_project(&[("x.rs", "content")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Open a file first
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Go to tree then back to center
    be.inject_key(KeyCode::F(2), KeyMod::default());
    be.inject_key(KeyCode::F(3), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert!(be.contains("content"));
}

#[test]
fn f4_focuses_right_slot() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    be.inject_key(KeyCode::F(4), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Right slot has Shell placeholder
    assert!(be.contains("[Shell]"));
}

#[test]
fn f5_toggles_zoom() {
    let dir = temp_project(&[("a.rs", "zoomed content")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Open file and focus center
    be.inject_key(KeyCode::Enter, KeyMod::default());
    be.inject_key(KeyCode::F(3), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Zoom (F5)
    be.inject_key(KeyCode::F(5), KeyMod::default());
    let screen = run_and_capture(&mut app, &mut be, 1);
    // When zoomed, content should be visible
    assert!(screen.contains("zoomed content"));
    // Unzoom
    be.inject_key(KeyCode::F(5), KeyMod::default());
    let screen2 = run_and_capture(&mut app, &mut be, 1);
    assert!(screen2.contains("a.rs")); // tree visible again
}
