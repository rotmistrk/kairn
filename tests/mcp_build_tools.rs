//! Integration tests for MCP Tier 5 — build/search tools.

mod helpers;

use helpers::{temp_project, TestHarness};

use kairn::mcp::commands::{McpAction, McpCommandQueue, McpRequest};
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
fn mcp_search_project_finds_matches() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() {\n    println!(\"hello\");\n}\n"),
        ("src/lib.rs", "pub fn hello() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::SearchProject {
            pattern: "hello".to_string(),
        },
    );
    assert!(result.is_ok(), "search_project failed: {result:?}");
    let val = result.unwrap();
    let matches = val["matches"].as_array().unwrap();
    assert!(matches.len() >= 2, "Expected at least 2 matches, got {}", matches.len());
}

#[test]
fn mcp_get_build_errors_returns_empty() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::GetBuildErrors);
    assert!(result.is_ok());
    let val = result.unwrap();
    let errors = val["errors"].as_array().unwrap();
    assert!(errors.is_empty(), "Expected no build errors initially");
}

#[test]
fn mcp_search_project_invalid_regex_returns_error() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.state.mcp_commands = Some(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::SearchProject {
            pattern: "[invalid".to_string(),
        },
    );
    assert!(result.is_err(), "Expected error for invalid regex");
}
