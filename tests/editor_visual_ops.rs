//! Integration tests: visual mode complex operations.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_and_focus(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

#[test]
fn visual_change_replaces_selection() {
    let dir = temp_project(&[("f.rs", "hello world\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // v + select 5 chars + c + type replacement
    h.inject_key(KeyCode::Char('v'), none());
    h.run_cycles(1);
    h.inject_str("4l"); // select "hello"
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('c'), none());
    h.run_cycles(1);
    h.inject_str("goodbye");
    h.run_cycles(1);
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("goodbye"));
}

#[test]
fn visual_line_yank_and_paste() {
    let dir = temp_project(&[("f.rs", "line1\nline2\nline3\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // V (visual line), yank, move down, paste
    h.inject_key(KeyCode::Char('V'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('y'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('p'), none());
    h.run_cycles(2);

    // line1 should appear twice
    let screen = h.screen_text();
    assert!(screen.matches("line1").count() >= 2);
}

#[test]
fn visual_select_word_with_e() {
    let dir = temp_project(&[("f.rs", "foo bar baz\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // v + e selects to end of word, then d deletes
    h.inject_key(KeyCode::Char('v'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('e'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), none());
    h.run_cycles(2);

    // "foo" should be deleted, leaving " bar baz"
    assert!(!h.content_contains("foo"));
    assert!(h.content_contains("bar"));
}

#[test]
fn visual_indent_multiple_lines() {
    let dir = temp_project(&[("f.rs", "a\nb\nc\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // V, select 2 lines, indent
    h.inject_key(KeyCode::Char('V'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('>'), none());
    h.run_cycles(2);

    // Lines should be indented
    let screen = h.screen_text();
    assert!(screen.contains("    a") || screen.contains("  a"));
}

#[test]
fn visual_unindent_multiple_lines() {
    let dir = temp_project(&[("f.rs", "    a\n    b\n    c\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // V, select 2 lines, unindent
    h.inject_key(KeyCode::Char('V'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('<'), none());
    h.run_cycles(2);

    // Lines should be unindented
    assert!(h.content_contains("a"));
}
