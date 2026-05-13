//! MCP snapshot collection tests.

mod helpers;

use helpers::{temp_project, TestHarness};
use std::sync::{Arc, Mutex};

use kairn::mcp::collect::collect_snapshot;
use kairn::mcp::snapshot::McpSnapshot;

#[test]
fn mcp_snapshot_collects_editor_tabs() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());

    // Open a file
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/main.rs"),
        ))),
    );
    h.run_cycles(2);

    // Collect snapshot directly from the desktop
    let desktop = h.program.desktop_mut();
    let lg = desktop
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<kairn::layout_group::LayoutGroup>())
        .expect("desktop is LayoutGroup");
    let snap = collect_snapshot(lg);

    assert!(!snap.tabs.is_empty(), "Expected tabs in snapshot");
    let editor_tab = snap.tabs.iter().find(|t| t.tab_type == "editor");
    assert!(editor_tab.is_some(), "Expected an editor tab in snapshot");

    // New fields: cursor, modified, order
    let tab = editor_tab.unwrap();
    assert!(tab.cursor.is_some(), "Editor tab should have cursor");
    assert_eq!(tab.cursor.as_ref().unwrap().line, 0);
    assert_eq!(tab.cursor.as_ref().unwrap().col, 0);
    assert!(!tab.modified, "Freshly opened file should not be modified");
    assert!(tab.path.is_some());
}

#[test]
fn mcp_snapshot_handler_updates_arc() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    h.state.mcp_snapshot = Some(Arc::clone(&snap));

    // Open file to generate commands
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("a.txt"),
        ))),
    );

    // Run many cycles — each dispatch_command call increments mcp_tick
    for _ in 0..25 {
        h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    }

    let locked = snap.lock().unwrap();
    assert!(
        !locked.tabs.is_empty(),
        "Expected snapshot populated after 20+ commands"
    );
}

#[test]
fn mcp_snapshot_no_panic_without_arc() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    // No mcp_snapshot set — should not panic
    h.run_cycles(25);
}

#[test]
fn mcp_snapshot_tracks_focus_and_slot() {
    let dir = temp_project(&[("a.txt", "aaa\n"), ("b.txt", "bbb\n")]);
    let mut h = TestHarness::new(dir.path());

    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE_FOCUS,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("a.txt"),
        ))),
    );
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE_FOCUS,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("b.txt"),
        ))),
    );
    h.run_cycles(2);

    let desktop = h.program.desktop_mut();
    let lg = desktop
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<kairn::layout_group::LayoutGroup>())
        .unwrap();
    let snap = collect_snapshot(lg);

    assert_eq!(snap.focused_slot, "center");
    // Exactly one tab should be focused
    let focused_count = snap.tabs.iter().filter(|t| t.focused).count();
    assert_eq!(focused_count, 1, "Exactly one tab should be focused");
}

#[test]
fn mcp_snapshot_terminal_index() {
    let dir = temp_project(&[]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    let desktop = h.program.desktop_mut();
    let lg = desktop
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<kairn::layout_group::LayoutGroup>())
        .unwrap();
    let snap = collect_snapshot(lg);

    // Terminal tabs should have sequential indices
    for (i, term) in snap.terminals.iter().enumerate() {
        assert_eq!(term.index, i);
    }
}

#[test]
fn mcp_tool_list_tabs_includes_new_fields() {
    use kairn::mcp::tools::handle_tool_call;
    use serde_json::Map;

    let dir = temp_project(&[("x.rs", "fn x() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    h.state.mcp_snapshot = Some(Arc::clone(&snap));

    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.path().join("x.rs")))),
    );
    // Run enough cycles to trigger snapshot update
    for _ in 0..25 {
        h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    }

    let result = handle_tool_call(&snap, "list_tabs", &Map::new()).unwrap();
    assert!(result.get("focused_slot").is_some());
    assert!(result.get("tabs").is_some());
    let tabs = result["tabs"].as_array().unwrap();
    let editor_tab = tabs.iter().find(|t| t["type"] == "editor").unwrap();
    assert!(editor_tab.get("focused").is_some());
    assert!(editor_tab.get("modified").is_some());
    assert!(editor_tab.get("order").is_some());
    assert!(editor_tab.get("cursor").is_some());
}

#[test]
fn mcp_tool_get_terminal_by_index() {
    use kairn::mcp::tools::handle_tool_call;
    use serde_json::{json, Map};

    let dir = temp_project(&[]);
    let mut h = TestHarness::new(dir.path());
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    h.state.mcp_snapshot = Some(Arc::clone(&snap));

    for _ in 0..25 {
        h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    }

    let locked = snap.lock().unwrap();
    if locked.terminals.is_empty() {
        return; // No terminals in test env (no PTY)
    }
    drop(locked);

    let mut args = Map::new();
    args.insert("index".to_string(), json!(0));
    let result = handle_tool_call(&snap, "get_terminal_content", &args);
    assert!(result.is_ok());
}
