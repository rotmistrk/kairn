//! Tests for MCP write tools (update_todo) and active tab visibility.

mod helpers;

use std::sync::{Arc, Mutex};

use serde_json::{json, Map};
use tempfile::TempDir;

use helpers::TestHarness;
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use kairn::mcp::snapshot::McpSnapshot;
use kairn::mcp::tools::handle_tool_call;
use txv_core::run::Waker;

fn temp_project_with_todo(items: &[(&str, bool)]) -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut todo = duir_core::TodoFile::new("Todo");
    todo.items = items
        .iter()
        .map(|(title, done)| {
            let mut item = duir_core::TodoItem::new(*title);
            if *done {
                item.completed = duir_core::model::Completion::Done;
            }
            item
        })
        .collect();
    let content = serde_json::to_string_pretty(&todo).unwrap();
    std::fs::write(dir.path().join(".kairn.todo"), content).unwrap();
    dir
}

fn read_todo(dir: &std::path::Path) -> duir_core::TodoFile {
    let content = std::fs::read_to_string(dir.join(".kairn.todo")).unwrap();
    serde_json::from_str(&content).unwrap()
}

/// Helper: push an MCP action and trigger drain via dispatch_command.
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
fn mcp_update_todo_toggle() {
    let dir = temp_project_with_todo(&[("task one", false), ("task two", true)]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::TodoToggle { path: vec![0] });
    assert!(result.is_ok(), "toggle failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(
        todo.items[0].completed,
        duir_core::model::Completion::Done,
        "first item should be done"
    );
    // Toggle second item (done -> open)
    let result = exec_mcp_action(&mut h, McpAction::TodoToggle { path: vec![1] });
    assert!(result.is_ok());
    let todo = read_todo(dir.path());
    assert_eq!(todo.items[1].completed, duir_core::model::Completion::Open);
}

#[test]
fn mcp_update_todo_add() {
    let dir = temp_project_with_todo(&[("existing", false)]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoAdd {
            path: vec![0],
            title: "new item".to_string(),
        },
    );
    assert!(result.is_ok(), "add failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items.len(), 2);
    assert_eq!(todo.items[1].title, "new item");
}

#[test]
fn mcp_update_todo_remove() {
    let dir = temp_project_with_todo(&[("keep", false), ("remove me", true)]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::TodoRemove { path: vec![1] });
    assert!(result.is_ok(), "remove failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items.len(), 1);
    assert_eq!(todo.items[0].title, "keep");
}

#[test]
fn mcp_update_todo_move_down() {
    let dir = temp_project_with_todo(&[("first", false), ("second", false)]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::TodoMoveDown { path: vec![0] });
    assert!(result.is_ok(), "move_down failed: {result:?}");

    let todo = read_todo(dir.path());
    assert_eq!(todo.items[0].title, "second");
    assert_eq!(todo.items[1].title, "first");
}

#[test]
fn mcp_list_tabs_includes_active_field() {
    let dir = temp_project_with_todo(&[]);
    let mut h = TestHarness::new(dir.path());
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    h.state.set_mcp_snapshot(Arc::clone(&snap));

    for _ in 0..25 {
        h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    }

    let result = handle_tool_call(&snap, None, "list_tabs", &Map::new()).unwrap();
    let tabs = result["tabs"].as_array().unwrap();
    let has_active = tabs.iter().any(|t| t["active"] == true);
    assert!(has_active, "Expected at least one active tab");
}

#[test]
fn mcp_update_todo_write_disabled_without_queue() {
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    let mut args = Map::new();
    args.insert("action".to_string(), json!("toggle"));
    args.insert("path".to_string(), json!([0]));
    let result = handle_tool_call(&snap, None, "update_todo", &args);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Write operations disabled");
}
