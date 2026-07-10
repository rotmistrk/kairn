//! Test: MCP tool confirm prompt should appear in status bar, not corrupt screen.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};

#[test]
fn confirm_prompt_renders_in_status_bar() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Push directly to program sink (simulates what handler_mcp does)
    let prompt = "MCP: allow 'execute_bash'? (command=ls) [y/n]".to_string();
    h.program
        .sink()
        .push_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ConfirmContext::McpToolConfirm)));
    h.program.sink().push_command(CM_CONFIRM, Some(Box::new(prompt)));
    h.run_cycles(5);

    let screen = h.screen_text();
    assert!(
        screen.contains("MCP: allow"),
        "confirm prompt should be visible, screen:\n{}",
        screen
    );
}

#[test]
fn confirm_prompt_long_does_not_overflow() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);

    // Very long prompt (longer than terminal width)
    let long_args = "a".repeat(200);
    let prompt = format!("MCP: allow 'tool'? ({long_args}) [y/n]");
    h.dispatch_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ConfirmContext::McpToolConfirm)));
    h.dispatch_command(CM_CONFIRM, Some(Box::new(prompt)));
    h.run_cycles(5);

    // Should not crash or corrupt — just check we get here
    let screen = h.screen_text();
    assert!(screen.len() > 0, "screen should render");

    // No prompt text in editor area
    let lines: Vec<&str> = screen.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if i < 23 {
            assert!(
                !line.contains("MCP: allow"),
                "prompt leaked to editor at line {i}: {line}"
            );
        }
    }
}
