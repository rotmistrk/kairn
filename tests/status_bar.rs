mod helpers;

use helpers::{run_and_capture, setup, temp_project};

#[test]
fn status_bar_shows_key_hints() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    let last_row = be.row(23);
    assert!(last_row.contains("F1:Help"));
    assert!(last_row.contains("^Q:Quit"));
}

#[test]
fn status_bar_shows_mx_hint() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    let last_row = be.row(23);
    assert!(last_row.contains("M-x"));
}
