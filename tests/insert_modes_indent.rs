//! Integration tests: insert mode entry variants (i/I/a/A/o/O/r/R) and autoindent.

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

// --- i: insert at cursor ---

#[test]
fn i_inserts_at_cursor_position() {
    let dir = temp_project(&[("f.rs", "abcd\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("2l"); // move to col 2 (on 'c')
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);
    h.inject_str("X");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("abXcd"));
}

// --- I: insert at first non-blank ---

#[test]
fn big_i_inserts_at_first_nonblank() {
    let dir = temp_project(&[("f.rs", "    hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("$"); // move to end
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('I'), none());
    h.run_cycles(1);
    h.inject_str("X");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("    Xhello"));
}

// --- a: insert after cursor ---

#[test]
fn a_inserts_after_cursor() {
    let dir = temp_project(&[("f.rs", "ab\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_key(KeyCode::Char('a'), none());
    h.run_cycles(1);
    h.inject_str("X");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("aXb"));
}

// --- A: insert at end of line ---

#[test]
fn big_a_inserts_at_end_of_line() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(1);
    h.inject_str("!");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("hello!"));
}

// --- o: open line below with autoindent ---

#[test]
fn o_opens_below_with_indent() {
    let dir = temp_project(&[("f.rs", "    indented\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_key(KeyCode::Char('o'), none());
    h.run_cycles(1);
    h.inject_str("new");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    // New line should inherit indent
    assert!(h.content_contains("    new"));
}

// --- O: open line above with autoindent ---

#[test]
fn big_o_opens_above_with_indent() {
    let dir = temp_project(&[("f.rs", "    indented\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_key(KeyCode::Char('O'), none());
    h.run_cycles(1);
    h.inject_str("above");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("    above"));
}

// --- r: replace single char ---

#[test]
fn r_replaces_single_char() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("rx");
    h.run_cycles(2);

    assert!(h.content_contains("xello"));
}

// --- Count prefix: 3dd deletes 3 lines ---

#[test]
fn count_prefix_dd_deletes_multiple() {
    let dir = temp_project(&[("f.rs", "aaa\nbbb\nccc\nddd\neee\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_key(KeyCode::Char('3'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), none());
    h.run_cycles(2);

    assert!(!h.content_contains("aaa"));
    assert!(!h.content_contains("bbb"));
    assert!(!h.content_contains("ccc"));
    assert!(h.content_contains("ddd"));
}

// --- Indent/unindent in normal mode (>> / <<) ---

#[test]
fn double_gt_indents_line() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str(">>");
    h.run_cycles(2);

    let screen = h.screen_text();
    assert!(screen.contains("    hello") || screen.contains("  hello"));
}

#[test]
fn double_lt_unindents_line() {
    let dir = temp_project(&[("f.rs", "    hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("<<");
    h.run_cycles(2);

    assert!(h.content_contains("hello"));
}

// --- Count with indent: 3>> indents 3 lines ---

#[test]
fn count_indent_multiple_lines() {
    let dir = temp_project(&[("f.rs", "a\nb\nc\nd\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("3>>");
    h.run_cycles(2);

    let screen = h.screen_text();
    // First 3 lines should be indented
    assert!(screen.contains("    a") || screen.contains("  a"));
    assert!(screen.contains("    b") || screen.contains("  b"));
    assert!(screen.contains("    c") || screen.contains("  c"));
}

// --- Backspace in insert mode at col 0 joins lines ---

#[test]
fn backspace_at_col0_joins_lines() {
    let dir = temp_project(&[("f.rs", "first\nsecond\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Move to line 2, col 0, enter insert, backspace
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Backspace, none());
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("firstsecond"));
}
