//! Integration tests for MCP Tier 2 — file/tab management tools.

mod helpers;

use helpers::{temp_project, TestHarness};
use std::sync::{Arc, Mutex};

use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
use kairn::mcp::snapshot::McpSnapshot;
use txv_core::run::Waker;

/// Push an MCP action and trigger drain via dispatch_command.
fn exec_mcp_action(h: &mut TestHarness, action: McpAction) -> Result<serde_json::Value, String> {
    let queue = h.state.mcp_commands.as_ref().unwrap();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    if let Ok(mut q) = queue.queue_handle().lock() {
        q.push_back(McpRequest { action, reply: tx });
    }
    h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    rx.recv().map_err(|e| e.to_string())?
}

#[test]
fn mcp_open_file_creates_tab() {
    let dir = temp_project(&[("src/lib.rs", "pub fn hello() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "src/lib.rs".to_string(),
        },
    );
    assert!(result.is_ok(), "open_file should succeed: {result:?}");

    // Verify tab exists via snapshot
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    h.state.mcp_snapshot = Some(Arc::clone(&snap));
    for _ in 0..25 {
        h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    }
    let locked = snap.lock().unwrap();
    let has_tab = locked.tabs.iter().any(|t| t.name.contains("lib.rs"));
    assert!(has_tab, "Expected lib.rs tab in snapshot");
}

#[test]
fn mcp_create_file_writes_and_opens() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::CreateFile {
            path: "new/file.txt".to_string(),
            content: "hello world".to_string(),
        },
    );
    assert!(result.is_ok(), "create_file should succeed: {result:?}");

    // Verify file on disk
    let path = dir.path().join("new/file.txt");
    assert!(path.exists(), "File should exist on disk");
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello world");
}

#[test]
fn mcp_close_tab_removes_tab() {
    let dir = temp_project(&[("a.txt", "aaa\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    // First open the file via MCP
    let result = exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "a.txt".to_string(),
        },
    );
    assert!(result.is_ok());

    // Now close it via MCP
    let result = exec_mcp_action(
        &mut h,
        McpAction::CloseTab {
            name: "a.txt".to_string(),
        },
    );
    assert!(result.is_ok(), "close_tab should succeed: {result:?}");
}

#[test]
fn mcp_open_file_not_found_returns_error() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "nonexistent.rs".to_string(),
        },
    );
    assert!(result.is_err(), "open_file should fail for missing file");
    assert!(result.unwrap_err().contains("not found"));
}
