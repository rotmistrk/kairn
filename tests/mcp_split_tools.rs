//! MCP Split tools — verifies split creation and close.

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
fn split_vertical_and_close() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 24);
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Open a file first
    exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "main.rs".to_string(),
        },
    )
    .unwrap();
    h.run_cycles(3);

    // Split vertical
    let result = exec_mcp_action(&mut h, McpAction::SplitVertical { file: None });
    assert!(result.is_ok(), "SplitVertical failed: {result:?}");
    assert_eq!(result.unwrap()["split"], "vertical");

    // Process the split command
    h.run_cycles(5);
    let screen = h.screen_text();
    // Vertical split should show divider
    assert!(
        screen.contains("│"),
        "should see vertical divider after split. Screen:\n{screen}"
    );

    // Close split
    let result = exec_mcp_action(&mut h, McpAction::SplitClose);
    assert!(result.is_ok(), "SplitClose failed: {result:?}");
    h.run_cycles(5);
}

#[test]
fn split_horizontal_creates_divider() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    println!(\"hi\");\n}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "main.rs".to_string(),
        },
    )
    .unwrap();
    h.run_cycles(3);

    let result = exec_mcp_action(&mut h, McpAction::SplitHorizontal { file: None });
    assert!(result.is_ok(), "SplitHorizontal failed: {result:?}");
    assert_eq!(result.unwrap()["split"], "horizontal");
    h.run_cycles(5);

    let screen = h.screen_text();
    assert!(
        screen.contains("────"),
        "should see horizontal divider after split. Screen:\n{screen}"
    );
}
