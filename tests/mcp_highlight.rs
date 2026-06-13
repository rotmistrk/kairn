//! MCP HighlightCode — verifies no crash when highlighting lines.

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
fn highlight_code_no_crash() {
    let content = (1..=10).map(|i| format!("line {i}\n")).collect::<String>();
    let dir = temp_project(&[("code.rs", &content)]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::HighlightCode {
            path: "code.rs".to_string(),
            ranges: vec![(3, 5)],
        },
    );
    assert!(result.is_ok(), "HighlightCode failed: {result:?}");
    let val = result.unwrap();
    assert_eq!(val["highlighted"], "code.rs");
    assert_eq!(val["ranges"], 1);
}
