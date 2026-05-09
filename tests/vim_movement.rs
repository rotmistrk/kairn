mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(
    be: &mut txv_core::run::MockBackend,
) {
    // File is first item in tree, Enter opens it, F3 focuses center
    be.inject_key(KeyCode::Enter, KeyMod::default());
    be.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn h_moves_left() {
    let dir = temp_project(&[("t.txt", "abcdef")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    // Move right then left
    be.inject_key(KeyCode::Char('l'), KeyMod::default());
    be.inject_key(KeyCode::Char('h'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 0)));
}

#[test]
fn l_moves_right() {
    let dir = temp_project(&[("t.txt", "abcdef")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('l'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 1)));
}

#[test]
fn j_moves_down() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('j'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((1, 0)));
}

#[test]
fn k_moves_up() {
    let dir = temp_project(&[("t.txt", "line1\nline2")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('j'), KeyMod::default());
    be.inject_key(KeyCode::Char('k'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 0)));
}

#[test]
fn w_moves_word_forward() {
    let dir = temp_project(&[("t.txt", "hello world")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('w'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 6)));
}

#[test]
fn b_moves_word_backward() {
    let dir = temp_project(&[("t.txt", "hello world")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('w'), KeyMod::default());
    be.inject_key(KeyCode::Char('b'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 0)));
}

#[test]
fn zero_moves_to_line_start() {
    let dir = temp_project(&[("t.txt", "hello world")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('w'), KeyMod::default());
    be.inject_key(KeyCode::Char('0'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 0)));
}

#[test]
fn dollar_moves_to_line_end() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('$'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 4)));
}

#[test]
fn gg_moves_to_file_start() {
    let dir = temp_project(&[("t.txt", "a\nb\nc\nd")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('G'), KeyMod::default());
    be.inject_key(KeyCode::Char('g'), KeyMod::default());
    be.inject_key(KeyCode::Char('g'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((0, 0)));
}

#[test]
fn g_moves_to_file_end() {
    let dir = temp_project(&[("t.txt", "a\nb\nc")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    open_file_and_focus(&mut be);
    be.inject_key(KeyCode::Char('G'), KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    assert_eq!(app.editor_cursor(), Some((2, 0)));
}
