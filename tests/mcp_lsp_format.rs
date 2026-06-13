//! MCP LspFormat — verifies no crash when LSP is not running.

mod helpers;

use helpers::{temp_project, TestHarness};
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
fn lsp_format_no_crash_without_lsp() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Open file first
    exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "main.rs".to_string(),
        },
    )
    .unwrap();
    h.run_cycles(2);

    // Dispatch LspFormat — LSP not running so should just dispatch command without panic
    let result = exec_mcp_action(
        &mut h,
        McpAction::LspFormat {
            name: "main.rs".to_string(),
        },
    );
    assert!(result.is_ok(), "LspFormat should succeed (just dispatches command)");
    assert_eq!(result.unwrap()["triggered"], "format");
}
