//! Integration tests for new MCP tools (git, lsp, undo/redo, eval_tcl).

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

// ─── eval_tcl ───────────────────────────────────────────────────────────────

#[test]
fn mcp_eval_tcl_returns_result() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::EvalTcl {
            script: "expr {2 + 3}".to_string(),
        },
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["result"], "5");
}

#[test]
fn mcp_eval_tcl_error_returns_err() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::EvalTcl {
            script: "error {deliberate failure}".to_string(),
        },
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Tcl error"));
}

#[test]
fn mcp_eval_tcl_set_and_get_variable() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    exec_mcp_action(
        &mut h,
        McpAction::EvalTcl {
            script: "set myvar hello".to_string(),
        },
    )
    .unwrap();

    let result = exec_mcp_action(
        &mut h,
        McpAction::EvalTcl {
            script: "set myvar".to_string(),
        },
    );
    assert_eq!(result.unwrap()["result"], "hello");
}

// ─── git operations ─────────────────────────────────────────────────────────

#[test]
fn mcp_git_stage_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::GitStage {
            file: "src/main.rs".to_string(),
        },
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["staged"], "src/main.rs");
}

#[test]
fn mcp_git_unstage_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::GitUnstage {
            file: "src/main.rs".to_string(),
        },
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["unstaged"], "src/main.rs");
}

#[test]
fn mcp_git_commit_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::GitCommit {
            message: "fix: resolve issue".to_string(),
        },
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["committed"], "fix: resolve issue");
}

// ─── LSP semantic ───────────────────────────────────────────────────────────

#[test]
fn mcp_lsp_hover_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::LspHover { name: String::new() });
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["triggered"], "hover");
}

#[test]
fn mcp_lsp_definition_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::LspDefinition { name: String::new() });
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["triggered"], "definition");
}

#[test]
fn mcp_lsp_references_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::LspReferences { name: String::new() });
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["triggered"], "references");
}

#[test]
fn mcp_lsp_rename_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(
        &mut h,
        McpAction::LspRename {
            name: String::new(),
            new_name: "new_fn".to_string(),
        },
    );
    assert!(result.is_ok());
    let val = result.unwrap();
    assert_eq!(val["triggered"], "rename");
    assert_eq!(val["new_name"], "new_fn");
}

#[test]
fn mcp_lsp_code_action_dispatches() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    let result = exec_mcp_action(&mut h, McpAction::LspCodeAction { name: String::new() });
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["triggered"], "code-action");
}

// ─── undo/redo ──────────────────────────────────────────────────────────────

#[test]
fn mcp_undo_on_active_editor() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    // Open a file first
    exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "src/main.rs".to_string(),
        },
    )
    .unwrap();
    h.run_cycles(2);

    let result = exec_mcp_action(
        &mut h,
        McpAction::Undo {
            name: "src/main.rs".to_string(),
        },
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["action"], "undo");
}

#[test]
fn mcp_redo_on_active_editor() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));

    exec_mcp_action(
        &mut h,
        McpAction::OpenFile {
            path: "src/main.rs".to_string(),
        },
    )
    .unwrap();
    h.run_cycles(2);

    let result = exec_mcp_action(
        &mut h,
        McpAction::Redo {
            name: "src/main.rs".to_string(),
        },
    );
    assert!(result.is_ok());
    assert_eq!(result.unwrap()["action"], "redo");
}

// ─── MCP tool definitions ───────────────────────────────────────────────────

#[test]
fn mcp_tool_definitions_include_new_tools() {
    let defs = kairn::mcp::tools_defs::tool_definitions();
    let tools = defs.as_array().unwrap();
    let names: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();

    assert!(names.contains(&"send_terminal_input"), "missing send_terminal_input");
    assert!(names.contains(&"git_ops"), "missing git_ops");
    assert!(names.contains(&"lsp_semantic"), "missing lsp_semantic");
    assert!(names.contains(&"undo_redo"), "missing undo_redo");
    assert!(names.contains(&"eval_tcl"), "missing eval_tcl");
}

#[test]
fn mcp_tool_definitions_have_valid_schemas() {
    let defs = kairn::mcp::tools_defs::tool_definitions();
    let tools = defs.as_array().unwrap();

    for tool in tools {
        let name = tool["name"].as_str().unwrap();
        assert!(tool["description"].is_string(), "{name} missing description");
        assert!(tool["inputSchema"].is_object(), "{name} missing inputSchema");
        assert_eq!(tool["inputSchema"]["type"], "object", "{name} schema type not object");
    }
}

// ─── MCP tool dispatch ──────────────────────────────────────────────────────

#[test]
fn mcp_handle_tool_call_unknown_returns_error() {
    use kairn::mcp::snapshot::McpSnapshot;
    use kairn::mcp::tools::handle_tool_call;
    use serde_json::Map;
    use std::sync::{Arc, Mutex};

    let snapshot = Arc::new(Mutex::new(McpSnapshot::default()));
    let args = Map::new();
    let result = handle_tool_call(&snapshot, None, "nonexistent_tool", &args);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Unknown tool"));
}
