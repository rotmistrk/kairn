//! Regression test for f3a7d00: Enter key should NOT accept completion.
//! Only Tab accepts; Enter should dismiss popup and insert a newline.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_LSP_COMPLETION;
use kairn::lsp::requests::{CompletionItem, CompletionKind};
use txv_core::event::{Event, KeyCode, KeyMod};

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

fn inject_completion(h: &mut TestHarness, items: Vec<CompletionItem>) {
    h.backend.inject(Event::Command {
        broadcast: false,
        id: CM_LSP_COMPLETION,
        data: Some(Box::new(items)),
    });
    h.run_cycles(1);
}

/// Enter dismisses the completion popup and inserts a newline (does NOT accept).
#[test]
fn enter_does_not_accept_completion() {
    let dir = temp_project(&[("f.rs", "fn f() { hel }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Move to after "hel" and enter insert mode
    h.inject_str("12l");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![CompletionItem::new(
            "hello_world",
            None,
            Some("hello_world".into()),
            CompletionKind::Other,
        )],
    );

    // Press Enter — should NOT accept "hello_world"
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);

    // "hello_world" should NOT appear — "hel" should remain with a newline inserted
    assert!(!h.content_contains("hello_world"), "Enter must not accept completion");
    assert!(h.content_contains("hel"), "original text 'hel' should remain");
}

/// Tab still accepts completion (positive control).
#[test]
fn tab_accepts_completion() {
    let dir = temp_project(&[("f.rs", "fn f() { hel }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("12l");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![CompletionItem::new(
            "hello_world",
            None,
            Some("hello_world".into()),
            CompletionKind::Other,
        )],
    );

    // Press Tab — should accept
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(2);

    assert!(h.content_contains("hello_world"), "Tab must accept completion");
}
