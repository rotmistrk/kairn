//! MCP SendTerminalInput — verifies no crash when sending input to terminal.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_NEW_SHELL;
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
fn send_terminal_input_no_crash() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Open a shell tab
    h.dispatch_command(CM_NEW_SHELL, None);
    h.run_cycles(3);

    // Send terminal input — may error (PTY not functional in test) but must not panic
    let result = exec_mcp_action(
        &mut h,
        McpAction::SendTerminalInput {
            name: "shell".to_string(),
            input: "echo hi\n".to_string(),
        },
    );
    // Either succeeds or returns an error string — no panic
    assert!(result.is_ok() || result.is_err());
}
