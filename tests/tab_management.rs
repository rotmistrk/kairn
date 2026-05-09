mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn new_file_creates_tab() {
    let dir = temp_project(&[("one.rs", "1")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert!(be.row(0).contains("one.rs"));
}

#[test]
fn tab_bar_shows_all_tabs() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    be.inject_key(KeyCode::F(2), KeyMod::default());
    be.inject_key(KeyCode::Down, KeyMod::default());
    be.inject_key(KeyCode::Enter, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    let tab_bar = be.row(0);
    assert!(tab_bar.contains("a.rs"));
    assert!(tab_bar.contains("b.rs"));
}
