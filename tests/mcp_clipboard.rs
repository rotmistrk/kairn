//! Test: MCP clipboard_copy, clipboard_paste, clipboard_list tools.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use txv_core::run::Waker;

fn exec_mcp(h: &mut TestHarness, action: McpAction) -> Result<serde_json::Value, String> {
    let queue = h.state.mcp_commands().as_ref().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    if let Ok(mut q) = queue.queue_handle().lock() {
        q.push_back(McpRequest::new(action, tx));
    }
    h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    rx.recv().map_err(|e| e.to_string())?
}

#[test]
fn mcp_clipboard_copy_paste_list() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    h.run_cycles(2);

    // Copy "hello" via MCP
    let result = exec_mcp(
        &mut h,
        McpAction::ClipboardCopy {
            text: "hello".to_string(),
            source: "test".to_string(),
        },
    );
    assert!(result.is_ok());

    // List clipboard — should contain "hello"
    let list = exec_mcp(&mut h, McpAction::ClipboardList).unwrap();
    let entries = list["entries"].as_array().unwrap();
    assert!(!entries.is_empty(), "clipboard should have entries");
    assert_eq!(entries[0]["first_line"], "hello");

    // Paste — should return "hello"
    let paste = exec_mcp(&mut h, McpAction::ClipboardPaste).unwrap();
    assert_eq!(paste["text"], "hello");
}
