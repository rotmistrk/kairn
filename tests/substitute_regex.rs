// === :1,.s/^/#/ substitution with regex and . address ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn substitute_caret_with_range_dot() {
    let dir = temp_project(&[("t.txt", "aaa\nbbb\nccc\nddd")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Move cursor to line 3 (0-indexed line 2)
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // :1,.s/^/#/ — comment lines 1 through current (3)
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("1,.s/^/#/\n");
    h.run_cycles(1);
    // Lines 1-3 should now start with #
    assert!(h.contains("#aaa"), "line 1 should be commented");
    assert!(h.contains("#bbb"), "line 2 should be commented");
    assert!(h.contains("#ccc"), "line 3 should be commented");
    // Line 4 should NOT be commented
    assert!(h.contains("ddd"));
    assert!(!h.contains("#ddd"), "line 4 should NOT be commented");
}

#[test]
fn substitute_dollar_regex() {
    let dir = temp_project(&[("t.txt", "hello\nworld")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // :%s/$/ END/ — append " END" to every line
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("%s/$/ END/\n");
    h.run_cycles(1);
    assert!(h.contains("hello END"), "expected 'hello END'");
    assert!(h.contains("world END"), "expected 'world END'");
}
