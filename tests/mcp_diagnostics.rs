//! MCP GetDiagnostics — verifies diagnostics are returned.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_DIAGNOSTIC, CM_OPEN_FILE_FOCUS};
use kairn::lsp::diagnostics::{Diagnostic, Severity};
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use txv_core::run::Waker;

fn exec_mcp_action(h: &mut TestHarness, action: McpAction) -> Result<serde_json::Value, String> {
    let queue = h.state.mcp_commands().as_ref().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    if let Ok(mut q) = queue.queue_handle().lock() {
        q.push_back(McpRequest::new(action, tx));
    }
    h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    rx.recv().map_err(|e| e.to_string())?
}

#[test]
fn get_diagnostics_returns_pushed_diagnostic() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    bad;\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Open file
    let path = dir.path().join("main.rs");
    let req = OpenFileRequest::new(path.clone());
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Push a diagnostic via CM_DIAGNOSTIC
    let uri = path.to_string_lossy().to_string();
    let diags = vec![Diagnostic::new(1, 4, 7, Severity::Error, "not found")];
    h.program
        .sink()
        .push_command(CM_DIAGNOSTIC, Some(Box::new((uri, diags))));
    h.run_cycles(3);

    // Get diagnostics via MCP
    let result = exec_mcp_action(
        &mut h,
        McpAction::GetDiagnostics {
            name: "main.rs".to_string(),
        },
    )
    .unwrap();
    let diag_arr = result["diagnostics"].as_array().unwrap();
    assert!(!diag_arr.is_empty(), "should have diagnostics");
    assert_eq!(diag_arr[0]["message"], "not found");
    assert_eq!(diag_arr[0]["severity"], "error");
    assert_eq!(diag_arr[0]["line"], 1);
}
