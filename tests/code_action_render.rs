//! Code action result rendering — verifies CM_SHELL_OUTPUT shows action titles.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS, CM_SHELL_OUTPUT};

#[test]
fn code_action_result_shows_output_tab() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);

    // Open a file
    let req = OpenFileRequest::new(dir.path().join("main.rs"));
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Simulate LSP code action result — actions joined by newline
    let actions_text = "Extract variable\nInline function\nAdd import".to_string();
    h.program
        .sink()
        .push_command(CM_SHELL_OUTPUT, Some(Box::new(actions_text)));
    h.run_cycles(5);

    // The output tab should show action titles
    let screen = h.screen_text();
    assert!(
        screen.contains("Extract variable"),
        "code action title should appear. Screen:\n{screen}"
    );
    assert!(
        screen.contains("Inline function"),
        "second action title should appear. Screen:\n{screen}"
    );
}
