//! MCP SetCursor — verifies cursor movement.

mod helpers;

use helpers::{cursor_at, temp_project, TestHarness};
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
fn set_cursor_moves_to_position() {
    let lines: String = (0..10).map(|i| format!("line {i} content\n")).collect();
    let dir = temp_project(&[("file.rs", &lines)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Open file
    exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "file.rs".to_string(),
        },
    )
    .unwrap();
    h.run_cycles(3);

    // Set cursor to line 5, col 3
    let result = exec_mcp_action(
        &mut h,
        McpAction::SetCursor {
            name: "file.rs".to_string(),
            line: 5,
            col: 3,
        },
    );
    assert!(result.is_ok(), "SetCursor failed: {result:?}");
    let val = result.unwrap();
    assert_eq!(val["cursor"]["line"], 5);
    assert_eq!(val["cursor"]["col"], 3);

    // Render and check cursor position
    h.run_cycles(3);
    if let Some((line, col)) = cursor_at(&h) {
        assert_eq!(line, 5, "cursor line should be 5");
        assert_eq!(col, 3, "cursor col should be 3");
    }
}
