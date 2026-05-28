//! Test: MCP add_subtree with empty path adds items at top level.

mod helpers;

use helpers::TestHarness;
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use serde_json::json;
use txv_core::run::Waker;

fn temp_project_with_todo(items: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut todo = duir_core::TodoFile::new("Todo");
    todo.items = items.iter().map(|t| duir_core::TodoItem::new(*t)).collect();
    let content = serde_json::to_string_pretty(&todo).unwrap();
    std::fs::write(dir.path().join(".kairn.todo"), content).unwrap();
    dir
}

fn read_todo(dir: &std::path::Path) -> duir_core::TodoFile {
    let content = std::fs::read_to_string(dir.join(".kairn.todo")).unwrap();
    serde_json::from_str(&content).unwrap()
}

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
fn add_subtree_empty_path_adds_top_level_items() {
    let dir = temp_project_with_todo(&["existing"]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let items = vec![
        json!({"title": "new-top-1"}),
        json!({"title": "new-top-2", "items": [{"title": "child-a"}]}),
    ];
    let result = exec_mcp_action(&mut h, McpAction::TodoAddSubtree { path: vec![], items });
    assert!(result.is_ok(), "add_subtree with empty path failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items.len(), 3, "should have 3 top-level items");
    assert_eq!(todo.items[0].title, "existing");
    assert_eq!(todo.items[1].title, "new-top-1");
    assert_eq!(todo.items[2].title, "new-top-2");
    assert_eq!(todo.items[2].items.len(), 1);
    assert_eq!(todo.items[2].items[0].title, "child-a");
}
