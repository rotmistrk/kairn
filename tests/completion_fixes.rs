//! Tests for completion fixes: popup position, full word replacement, cursor placement.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_LSP_COMPLETION;
use kairn::lsp::requests::CompletionItem;
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
        id: CM_LSP_COMPLETION,
        data: Some(Box::new(items)),
    });
    h.run_cycles(1);
}

fn accept(h: &mut TestHarness) {
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(2);
}

/// Cursor in middle of word: GetLo|gGroup → accept GetLogLevel → replaces entire word.
#[test]
fn completion_replaces_entire_word_under_cursor() {
    let dir = temp_project(&[("f.rs", "fn f() { GetLogGroup }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position cursor on 'g' of "GetLogGroup" (col 14) using normal mode
    // "fn f() { GetLogGroup }" — 'G' is at col 9, 'g' (second) at col 14
    h.inject_str("14l"); // move to col 14
    h.run_cycles(1);

    // Enter insert mode at cursor position (i = insert before current char)
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![CompletionItem {
            label: "GetLogLevel".into(),
            detail: None,
            insert_text: Some("GetLogLevel".into()),
        }],
    );
    accept(&mut h);

    // Should have GetLogLevel, NOT GetLogGroupGetLogLevel or GetLoGetLogLevel
    assert!(h.content_contains("GetLogLevel"));
    assert!(!h.content_contains("GetLogGroup"));
}

/// Cursor at end of word: pri| → accept println → replaces "pri" with "println".
#[test]
fn completion_replaces_prefix_at_end_of_word() {
    let dir = temp_project(&[("f.rs", "fn f() { pri }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position after "pri" (col 12) — "fn f() { pri }" → 'p' at col 9, end of "pri" at col 12
    h.inject_str("12l"); // on space after "pri"
    h.run_cycles(1);
    // 'i' inserts before current char → cursor at col 12 (after "pri")
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![CompletionItem {
            label: "println".into(),
            detail: None,
            insert_text: Some("println".into()),
        }],
    );
    accept(&mut h);

    assert!(h.content_contains("println"));
    assert!(!h.content_contains("priprintln"));
}

/// Popup position: should appear near cursor, not shifted right by parent offset.
#[test]
fn completion_popup_draws_near_cursor() {
    let dir = temp_project(&[("f.rs", "fn f() { pri }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position on space after "pri" and enter insert mode
    h.inject_str("12l");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![
            CompletionItem {
                label: "println".into(),
                detail: None,
                insert_text: None,
            },
            CompletionItem {
                label: "print".into(),
                detail: None,
                insert_text: None,
            },
        ],
    );
    h.run_cycles(1);

    // The popup should render "println" somewhere on screen (not off-screen to the right)
    assert!(h.content_contains("println"));
}

/// After accepting completion, cursor is at end of inserted text.
#[test]
fn cursor_at_end_after_completion() {
    let dir = temp_project(&[("f.rs", "fn f() { hel }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position after "hel" (col 12) and enter insert mode
    h.inject_str("12l");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    inject_completion(
        &mut h,
        vec![CompletionItem {
            label: "hello_world".into(),
            detail: None,
            insert_text: Some("hello_world".into()),
        }],
    );
    accept(&mut h);

    // Type a char — it should appear right after "hello_world"
    h.inject_str("X");
    h.run_cycles(2);

    assert!(h.content_contains("hello_worldX"));
}
