// === Dot repeat after insert session ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn dot_repeats_x_delete() {
    let dir = temp_project(&[("t.txt", "abcdef")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // x deletes first char, . repeats
    h.inject_key(KeyCode::Char('x'), KeyMod::default());
    h.inject_key(KeyCode::Char('.'), KeyMod::default());
    h.inject_key(KeyCode::Char('.'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("def"), "expected 3 chars deleted by x..");
    assert!(!h.contains("abc"));
}

#[test]
fn dot_repeats_dd() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3\nline4")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // dd deletes line, . repeats
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.inject_key(KeyCode::Char('.'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.contains("line1"));
    assert!(!h.contains("line2"));
    assert!(h.contains("line3"));
}
