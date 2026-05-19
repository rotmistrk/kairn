//! Integration tests: gs (highlight word under cursor / goto-show).

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
fn gs_does_not_crash_without_lsp() {
    let dir = temp_project(&[("src/main.rs", "fn hello() {\n    world();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // Position on "hello" and press gs
    h.inject_str("gs");
    h.run_cycles(2);

    // Editor should still be functional
    assert!(h.content_contains("hello"));
}

#[test]
fn gs_clears_on_next_keypress() {
    let dir = temp_project(&[("src/main.rs", "fn hello() {\n    world();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    h.inject_str("gs");
    h.run_cycles(2);

    // Any movement should clear highlight
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(2);

    // Editor still works after clearing
    assert!(h.content_contains("world"));
}

#[test]
fn gs_on_empty_line_no_crash() {
    let dir = temp_project(&[("src/main.rs", "\n\nfn foo() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // Cursor is on empty line 1
    h.inject_str("gs");
    h.run_cycles(2);

    assert!(h.content_contains("foo"));
}
