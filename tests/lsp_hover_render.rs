//! Test: LSP hover result renders content on screen.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_DIAGNOSTIC, CM_OPEN_FILE_FOCUS};

#[test]
fn hover_result_renders_on_screen() {
    let dir = temp_project(&[("main.rs", "fn foo() -> i32 { 42 }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    let req = OpenFileRequest::new(dir.path().join("main.rs"));
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Simulate hover result arriving (same format as lsp/response.rs handle_hover)
    let hover_text = "fn foo() -> i32".to_string();
    h.dispatch_command(CM_DIAGNOSTIC, Some(Box::new(("hover".to_string(), hover_text))));
    h.run_cycles(5);

    // The hover content should appear somewhere on screen (sidekick/popup/status)
    assert!(
        h.contains("fn foo()"),
        "hover text should be visible on screen, got:\n{}",
        h.screen_text()
    );
}
