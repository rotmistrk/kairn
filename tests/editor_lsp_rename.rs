//! Integration tests: gR (LSP rename) flow.

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
fn gr_rename_keystroke_no_crash() {
    let dir = temp_project(&[("src/main.rs", "fn hello() {\n    hello();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // Position on "hello" and press gR
    h.inject_str("w"); // move to "hello"
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('g'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('R'), none());
    h.run_cycles(3);

    // Without LSP, rename won't complete but shouldn't crash
    assert!(h.content_contains("hello"));
}

#[test]
fn lsp_rename_command_no_crash() {
    let dir = temp_project(&[("src/main.rs", "fn foo() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // Trigger rename via command dispatch (simulates M-x lsp-rename)
    h.dispatch_command(kairn::commands::CM_LSP_RENAME, Some(Box::new("bar".to_string())));
    h.run_cycles(3);

    // Without LSP server, nothing happens but no crash
    assert!(h.content_contains("foo"));
}

#[test]
fn gr_on_empty_file_no_crash() {
    let dir = temp_project(&[("empty.rs", "\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "empty.rs");

    h.inject_str("gR");
    h.run_cycles(3);

    // Should not crash
    assert!(h.screen_text().len() > 0);
}
