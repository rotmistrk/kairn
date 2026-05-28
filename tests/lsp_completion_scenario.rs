//! Scenario tests for LSP completion and error handling.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{CM_LSP_COMPLETION, CM_OPEN_FILE};
use kairn::lsp::requests::CompletionItem;
use txv_core::event::{Event, KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

/// Open a file and focus the editor panel.
fn open_file(h: &mut TestHarness, path: std::path::PathBuf) {
    h.dispatch_command(
        CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(path))),
    );
    h.run_cycles(2);
    // Focus center (editor)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

/// Send a command event directly to the focused view (bypasses main handler).
fn send_to_view(h: &mut TestHarness, id: u16, data: Option<Box<dyn std::any::Any + Send>>) {
    let event = Event::Command { id, data };
    h.backend.inject(event);
    h.run_cycles(1);
}

/// Test 1: Completion prefix removal — accepting "println" when prefix is "pri"
/// should produce "println", not "priprintln".
#[test]
fn completion_prefix_removal() {
    let dir = temp_project(&[("src/main.rs", "fn main() { pri }")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    open_file(&mut h, dir.path().join("src/main.rs"));

    // Enter insert mode
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);

    // Position cursor after "pri": End then Left twice (skip " }")
    h.inject_key(KeyCode::End, none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Left, none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Left, none());
    h.run_cycles(1);

    // Send completion items to the editor view
    let items: Vec<CompletionItem> = vec![CompletionItem::new(
        "println",
        None,
        Some("println".into()),
        kairn::lsp::requests::CompletionKind::Other,
    )];
    send_to_view(&mut h, CM_LSP_COMPLETION, Some(Box::new(items)));
    h.run_cycles(1);

    // Accept completion with Tab
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(2);

    // Verify buffer contains "println" and NOT "priprintln"
    assert!(h.content_contains("println"));
    assert!(!h.content_contains("priprintln"));
}

/// Test 2: Completion with insert_text different from label.
/// When prefix is "in" and we accept item with label "inner : State" and insertText "inner",
/// the buffer should have "inner", not "ininner : State".
#[test]
fn completion_insert_text_differs_from_label() {
    let dir = temp_project(&[("src/lib.rs", "fn foo() { in }")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    open_file(&mut h, dir.path().join("src/lib.rs"));

    // Enter insert mode, position after "in"
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::End, none());
    h.run_cycles(1);
    // "fn foo() { in }" — back up 2 (space + '}')
    h.inject_key(KeyCode::Left, none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Left, none());
    h.run_cycles(1);

    // Send completion with label != insert_text
    let items: Vec<CompletionItem> = vec![CompletionItem::new(
        "inner : State",
        Some("field".into()),
        Some("inner".into()),
        kairn::lsp::requests::CompletionKind::Other,
    )];
    send_to_view(&mut h, CM_LSP_COMPLETION, Some(Box::new(items)));
    h.run_cycles(1);

    // Accept
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(2);

    // Should have "inner", not "ininner" or "ininner : State"
    assert!(h.content_contains("inner"));
    assert!(!h.content_contains("ininner"));
}

/// Test 3: LSP error shown in status bar.
/// When an LSP error response arrives, the error message should appear on screen.
#[test]
fn lsp_error_shown_in_status() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}")]);
    // Use wider terminal so the status bar has room for the message
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);

    open_file(&mut h, dir.path().join("src/main.rs"));

    // Inject status message as a command event into the backend queue.
    // This flows through run_cycles → group.dispatch → StatusBar → MessageItem,
    // matching the real app flow when poll_lsp emits an error.
    use txv_core::message::Message;
    let msg = Message::error("lsp", "not ready");
    h.backend.inject(Event::Command {
        id: txv_widgets::CM_STATUS_MESSAGE,
        data: Some(Box::new(msg)),
    });
    h.run_cycles(1);

    // The error text should appear in the status bar
    assert!(h.contains("not ready"));
}
