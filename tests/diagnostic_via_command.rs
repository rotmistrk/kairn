//! Scenario: CM_DIAGNOSTIC command propagates markers to editor view.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_DIAGNOSTIC, CM_OPEN_FILE_FOCUS};
use kairn::lsp::diagnostics::{Diagnostic, Severity};

#[test]
fn diagnostic_marker_appears_via_command() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    bad;\n}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 60, 10);
    h.run_cycles(1);

    // Open the file
    let path = dir.path().join("main.rs");
    let req = OpenFileRequest::new(path.clone());
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Send CM_DIAGNOSTIC with a path string (as LSP handler sends it)
    let uri = path.to_string_lossy().to_string();
    let diags = vec![Diagnostic::new(1, 4, 7, Severity::Error, "not found")];
    h.program
        .sink()
        .push_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
    h.run_cycles(3);

    // The diagnostic marker ● should appear in the right gutter column
    assert!(
        h.content_contains("●"),
        "diagnostic ● marker should appear after CM_DIAGNOSTIC"
    );
}
