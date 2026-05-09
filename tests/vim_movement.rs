mod helpers;

use helpers::{cursor_at, temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn h_moves_left() {
    let dir = temp_project(&[("t.txt", "abcdef")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 0)));
}

#[test]
fn l_moves_right() {
    let dir = temp_project(&[("t.txt", "abcdef")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 1)));
}

#[test]
fn j_moves_down() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((1, 0)));
}

#[test]
fn k_moves_up() {
    let dir = temp_project(&[("t.txt", "line1\nline2")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 0)));
}

#[test]
fn w_moves_word_forward() {
    let dir = temp_project(&[("t.txt", "hello world")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('w'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 6)));
}

#[test]
fn b_moves_word_backward() {
    let dir = temp_project(&[("t.txt", "hello world")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('w'), KeyMod::default());
    h.inject_key(KeyCode::Char('b'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 0)));
}

#[test]
fn zero_moves_to_line_start() {
    let dir = temp_project(&[("t.txt", "hello world")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('w'), KeyMod::default());
    h.inject_key(KeyCode::Char('0'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 0)));
}

#[test]
fn dollar_moves_to_line_end() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('$'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 4)));
}

#[test]
fn gg_moves_to_file_start() {
    let dir = temp_project(&[("t.txt", "a\nb\nc\nd")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 0)));
}

#[test]
fn g_moves_to_file_end() {
    let dir = temp_project(&[("t.txt", "a\nb\nc")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((2, 0)));
}
