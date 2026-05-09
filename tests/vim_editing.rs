mod helpers;

use helpers::{cursor_at, run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(be: &mut txv_core::run::MockBackend) {
    be.inject_key(KeyCode::Enter, KeyMod::default());
    be.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn i_enters_insert_mode() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('i'), KeyMod::default());
    be.inject_key(KeyCode::Char('X'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert!(be.contains("Xhello"));
}

#[test]
fn typing_inserts_text() {
    let dir = temp_project(&[("t.txt", "ab")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('i'), KeyMod::default());
    be.inject_str("XY");
    run_and_capture(&mut app, &mut be, 1);
    assert!(be.contains("XYab"));
}

#[test]
fn esc_returns_to_normal() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('i'), KeyMod::default());
    be.inject_key(KeyCode::Esc, KeyMod::default());
    // In normal mode, 'l' moves right instead of inserting
    be.inject_key(KeyCode::Char('l'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(cursor_at(&be), Some((0, 1)));
}

#[test]
fn x_deletes_char() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('x'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert!(be.contains("ello"));
}

#[test]
fn dd_deletes_line() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('d'), KeyMod::default());
    be.inject_key(KeyCode::Char('d'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert!(!be.contains("line1"));
    assert!(be.contains("line2"));
}

#[test]
fn u_undoes_last_edit() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('x'), KeyMod::default());
    be.inject_key(KeyCode::Char('u'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert!(be.contains("hello"));
}

#[test]
fn p_pastes_after() {
    let dir = temp_project(&[("t.txt", "line1\nline2")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    // yy then p
    be.inject_key(KeyCode::Char('y'), KeyMod::default());
    be.inject_key(KeyCode::Char('y'), KeyMod::default());
    be.inject_key(KeyCode::Char('p'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Should have line1 duplicated
    let screen = be.screen_text();
    assert_eq!(screen.matches("line1").count(), 2);
}

#[test]
fn yy_yanks_line() {
    let dir = temp_project(&[("t.txt", "first\nsecond")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('y'), KeyMod::default());
    be.inject_key(KeyCode::Char('y'), KeyMod::default());
    // Move down and paste
    be.inject_key(KeyCode::Char('j'), KeyMod::default());
    be.inject_key(KeyCode::Char('p'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    let screen = be.screen_text();
    assert_eq!(screen.matches("first").count(), 2);
}
