//! Tests for MCP todo actions related to completion state.
//!
//! Parent completion is computed from children, so:
//! - set_completed on parent → error
//! - toggle on parent → error
//! - set_completed on leaf → propagates to parent
//! - add/remove → reconciles parent state

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use txv_core::run::Waker;

fn create_nested_todo(dir: &std::path::Path, structure: &str) {
    std::fs::write(dir.join(".kairn.todo"), structure).unwrap();
}

fn read_todo(dir: &std::path::Path) -> duir_core::TodoFile {
    let content = std::fs::read_to_string(dir.join(".kairn.todo")).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn focus_todo(h: &mut TestHarness) {
    use kairn::handler::downcast_desktop;
    use kairn::slots::{focus_tab_by_title, SlotId};
    let desktop = h.program.desktop_mut();
    if let Some(d) = downcast_desktop(desktop) {
        focus_tab_by_title(d, SlotId::Left, "Todo");
        d.focus_panel(SlotId::Left as usize);
    }
    h.run_cycles(2);
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

const PARENT_CHILD_TODO: &str = r#"{
    "version": "2.0", "title": "Todo",
    "items": [{
        "title": "Parent", "completed": "Open", "important": false, "folded": false,
        "items": [
            {"title": "Child", "completed": "Open", "important": false, "folded": false, "items": []}
        ]
    }]
}"#;

#[test]
fn mcp_set_completed_fails_on_parent() {
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), PARENT_CHILD_TODO);

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    focus_todo(&mut h);

    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoSetCompleted {
            path: vec![0],
            state: "done".to_string(),
        },
    );

    assert!(result.is_err(), "set_completed on parent should fail");
    let err = result.unwrap_err();
    assert!(err.contains("children") || err.contains("parent"), "Error: {err}");
}

#[test]
fn mcp_set_completed_works_on_leaf() {
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), PARENT_CHILD_TODO);

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    focus_todo(&mut h);

    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoSetCompleted {
            path: vec![0, 0],
            state: "done".to_string(),
        },
    );

    assert!(result.is_ok(), "set_completed on leaf should succeed: {result:?}");

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].items[0].completed, duir_core::model::Completion::Done);
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Done);
}

#[test]
fn mcp_toggle_on_parent_fails() {
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), PARENT_CHILD_TODO);

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    focus_todo(&mut h);

    let result = exec_mcp_action(&mut h, McpAction::TodoToggle { path: vec![0] });

    // Should fail (parent completion is computed)
    assert!(result.is_err(), "toggle on parent should fail");

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Open);
}

#[test]
fn add_child_triggers_reconciliation() {
    let todo = r#"{
        "version": "2.0", "title": "Todo",
        "items": [{
            "title": "Parent", "completed": "Done", "important": false, "folded": false,
            "items": [
                {"title": "Existing", "completed": "Done", "important": false, "folded": false, "items": []}
            ]
        }]
    }"#;
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), todo);

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    focus_todo(&mut h);

    let result = exec_mcp_action(
        &mut h,
        McpAction::TodoAdd {
            path: vec![0, 0],
            title: "New child".to_string(),
        },
    );
    assert!(result.is_ok(), "TodoAdd should succeed: {result:?}");

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Partial);
}

#[test]
fn remove_child_triggers_reconciliation() {
    let todo = r#"{
        "version": "2.0", "title": "Todo",
        "items": [{
            "title": "Parent", "completed": "Partial", "important": false, "folded": false,
            "items": [
                {"title": "Done child", "completed": "Done", "important": false, "folded": false, "items": []},
                {"title": "Open child", "completed": "Open", "important": false, "folded": false, "items": []}
            ]
        }]
    }"#;
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), todo);

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    focus_todo(&mut h);

    let result = exec_mcp_action(&mut h, McpAction::TodoRemove { path: vec![0, 1] });
    assert!(result.is_ok(), "TodoRemove should succeed: {result:?}");

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Done);
}
