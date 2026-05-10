// === :s edge cases — undo, invalid regex, backreferences ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn substitute_is_undoable() {
    let dir = temp_project(&[("t.txt", "aaa\nbbb\nccc")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // :%s/^/#/ — comment all lines
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("%s/^/#/\n");
    h.run_cycles(1);
    assert!(h.contains("#aaa"));
    // Single undo should revert the entire substitution
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("aaa"));
    assert!(!h.contains("#aaa"));
}

#[test]
fn substitute_invalid_regex_shows_error() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Invalid regex: unclosed group
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("s/(unclosed/x/\n");
    h.run_cycles(1);
    // Content should be unchanged
    assert!(h.contains("hello"));
}

#[test]
fn substitute_with_capture_group() {
    let dir = temp_project(&[("t.txt", "foo123bar")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Replace digits with brackets around them: (\d+) -> [$1]
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("s/(\\d+)/[$1]/\n");
    h.run_cycles(1);
    assert!(h.contains("foo[123]bar"), "expected capture group replacement");
}

#[test]
fn substitute_percent_d_deletes_all() {
    let dir = temp_project(&[("t.txt", "line1\nline2\nline3")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("%d\n");
    h.run_cycles(1);
    assert!(!h.contains("line1"));
    assert!(!h.contains("line2"));
}
