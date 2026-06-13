//! MCP Todo extended operations — priority, note, promote, demote, completed.

mod helpers;

use helpers::TestHarness;
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use txv_core::run::Waker;

fn temp_project_with_todo(items: &[&str]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut todo = duir_core::TodoFile::new("Todo");
    todo.items = items.iter().map(|t| duir_core::TodoItem::new(*t)).collect();
    // Add a child to first item so promote/demote can work
    if todo.items.len() >= 2 {
        let child = todo.items.remove(1);
        todo.items[0].items.push(child);
    }
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
fn todo_add_and_set_priority() {
    let dir = temp_project_with_todo(&["existing", "child"]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Add a new item
    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoAdd {
            path: vec![0],
            title: "new task".to_string(),
        },
    );
    assert!(result.is_ok(), "TodoAdd failed: {result:?}");

    // Set priority on the new item (now at index 1)
    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoSetPriority {
            path: vec![1],
            priority: 3,
        },
    );
    assert!(result.is_ok(), "TodoSetPriority failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items[1].priority, Some(3));
}

#[test]
fn todo_set_note() {
    let dir = temp_project_with_todo(&["task", "child"]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoSetNote {
            path: vec![0],
            note: "my note content".to_string(),
        },
    );
    assert!(result.is_ok(), "TodoSetNote failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items[0].note, "my note content");
}

#[test]
fn todo_promote_demote() {
    let dir = temp_project_with_todo(&["parent", "child"]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // The child is at path [0, 0] — promote it to top level
    let result = exec_mcp_action(&mut h, McpAction::TodoPromote { path: vec![0, 0] });
    assert!(result.is_ok(), "TodoPromote failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items.len(), 2, "child should now be at top level");

    // Demote second item back under first
    let result = exec_mcp_action(&mut h, McpAction::TodoDemote { path: vec![1] });
    assert!(result.is_ok(), "TodoDemote failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items.len(), 1, "should be back to 1 top-level");
    assert_eq!(todo.items[0].items.len(), 1, "child should be nested again");
}

#[test]
fn todo_set_completed() {
    let dir = temp_project_with_todo(&["task", "child"]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoSetCompleted {
            path: vec![0],
            state: "done".to_string(),
        },
    );
    assert!(result.is_ok(), "TodoSetCompleted failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items[0].completed, duir_core::model::Completion::Done);
}
