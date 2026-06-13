//! MCP ListRoots/AddRoot/RemoveRoot — verifies workspace root management.

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
fn mcp_roots_lifecycle() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let extra = tempfile::tempdir().unwrap();
    std::fs::write(extra.path().join("lib.rs"), "pub fn hi() {}").unwrap();
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // List roots — should have 1 (the project root)
    let result = exec_mcp_action(&mut h, McpAction::ListRoots).unwrap();
    let roots = result["roots"].as_array().unwrap();
    assert_eq!(roots.len(), 1, "should start with 1 root");

    // Add a root
    let extra_path = extra.path().to_string_lossy().to_string();
    let result = exec_mcp_action(
        &mut h,
        McpAction::AddRoot {
            path: extra_path.clone(),
        },
    )
    .unwrap();
    assert_eq!(result["added"], true);

    // List again — should have 2
    let result = exec_mcp_action(&mut h, McpAction::ListRoots).unwrap();
    let roots = result["roots"].as_array().unwrap();
    assert_eq!(roots.len(), 2, "should have 2 roots after add");

    // Remove the added root
    let result = exec_mcp_action(&mut h, McpAction::RemoveRoot { path: extra_path }).unwrap();
    assert_eq!(result["removed"], true);

    // Back to 1 root
    let result = exec_mcp_action(&mut h, McpAction::ListRoots).unwrap();
    let roots = result["roots"].as_array().unwrap();
    assert_eq!(roots.len(), 1, "should be back to 1 root after remove");
}
