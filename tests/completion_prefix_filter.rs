//! Regression test for dbc3518: Completion items should be filtered by the
//! typed prefix. Items not matching the prefix should be hidden.

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

/// Only items whose insert_text starts with the typed prefix should appear.
#[test]
fn completion_filters_by_typed_prefix() {
    let dir = temp_project(&[("f.rs", "fn f() { prin }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position after "prin" (col 13) and enter insert mode
    h.inject_str("13l");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    // Send items — only "println" matches prefix "prin", "format" does not
    inject_completion(
        &mut h,
        vec![
            CompletionItem::new("println", None, Some("println".into()), CompletionKind::Other),
            CompletionItem::new("format", None, Some("format".into()), CompletionKind::Other),
            CompletionItem::new("print", None, Some("print".into()), CompletionKind::Other),
        ],
    );
    h.run_cycles(2);

    // "println" and "print" match prefix "prin" — should be visible
    assert!(h.content_contains("println"), "matching item 'println' should appear");
    assert!(h.content_contains("print"), "matching item 'print' should appear");
    // "format" does NOT match prefix "prin" — should be filtered out
    assert!(
        !h.content_contains("format"),
        "non-matching item 'format' should be filtered out"
    );
}

/// Empty prefix shows all items.
#[test]
fn completion_empty_prefix_shows_all() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position at the space (col 9) between "{ " and " }" and enter insert mode
    h.inject_str("9l");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![
            CompletionItem::new("println", None, Some("println".into()), CompletionKind::Other),
            CompletionItem::new("format", None, Some("format".into()), CompletionKind::Other),
        ],
    );
    h.run_cycles(2);

    // Both should be visible when prefix is empty
    assert!(h.content_contains("println"));
    assert!(h.content_contains("format"));
}
