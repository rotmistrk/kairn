mod helpers;

use helpers::{cursor_at, temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn i_enters_insert_mode() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("Xhello"));
}

#[test]
fn typing_inserts_text() {
    let dir = temp_project(&[("t.txt", "ab")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_str("XY");
    h.run_cycles(1);
    assert!(h.contains("XYab"));
}

#[test]
fn esc_returns_to_normal() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(1);
    assert_eq!(cursor_at(&h), Some((0, 1)));
}

#[test]
fn x_deletes_char() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('x'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("ello"));
}

#[test]
fn dd_deletes_line() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.contains("line1"));
    assert!(h.contains("line2"));
}

#[test]
fn u_undoes_last_edit() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('x'), KeyMod::default());
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("hello"));
}

#[test]
fn p_pastes_after() {
    let dir = temp_project(&[("t.txt", "line1\nline2")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    assert_eq!(screen.matches("line1").count(), 2);
}

#[test]
fn yy_yanks_line() {
    let dir = temp_project(&[("t.txt", "first\nsecond")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    assert_eq!(screen.matches("first").count(), 2);
}
