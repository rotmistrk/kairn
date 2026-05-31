//! MCP tool handlers — terminal, git, LSP, undo/redo, eval.

use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};

pub fn tool_send_terminal_input(
    cmd_queue: Option<&McpCommandQueue>,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).ok_or("Missing 'name'")?;
    let input = args.get("input").and_then(Value::as_str).ok_or("Missing 'input'")?;
    queue.send(McpAction::SendTerminalInput {
        name: name.to_string(),
        input: input.to_string(),
    })?;
    Ok(json!({"sent": true, "target": name}))
}

pub fn tool_git_ops(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let action = args.get("action").and_then(Value::as_str).ok_or("Missing 'action'")?;
    match action {
        "stage" => {
            let file = args.get("file").and_then(Value::as_str).ok_or("Missing 'file'")?;
            queue.send(McpAction::GitStage { file: file.to_string() })
        }
        "unstage" => {
            let file = args.get("file").and_then(Value::as_str).ok_or("Missing 'file'")?;
            queue.send(McpAction::GitUnstage { file: file.to_string() })
        }
        "commit" => {
            let message = args.get("message").and_then(Value::as_str).ok_or("Missing 'message'")?;
            queue.send(McpAction::GitCommit {
                message: message.to_string(),
            })
        }
        _ => Err(format!("Unknown git action: {action}")),
    }
}

pub fn tool_lsp_semantic(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let action = args.get("action").and_then(Value::as_str).ok_or("Missing 'action'")?;
    let name = args.get("name").and_then(Value::as_str).unwrap_or("").to_string();
    match action {
        "hover" => queue.send(McpAction::LspHover { name }),
        "definition" => queue.send(McpAction::LspDefinition { name }),
        "references" => queue.send(McpAction::LspReferences { name }),
        "rename" => {
            let new_name = args
                .get("new_name")
                .and_then(Value::as_str)
                .ok_or("Missing 'new_name'")?;
            queue.send(McpAction::LspRename {
                name,
                new_name: new_name.to_string(),
            })
        }
        "code-action" => queue.send(McpAction::LspCodeAction { name }),
        _ => Err(format!("Unknown LSP action: {action}")),
    }
}

pub fn tool_undo_redo(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let action = args.get("action").and_then(Value::as_str).ok_or("Missing 'action'")?;
    let name = args.get("name").and_then(Value::as_str).unwrap_or("").to_string();
    match action {
        "undo" => queue.send(McpAction::Undo { name }),
        "redo" => queue.send(McpAction::Redo { name }),
        _ => Err(format!("Unknown action: {action}")),
    }
}

pub fn tool_eval_tcl(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let script = args.get("script").and_then(Value::as_str).ok_or("Missing 'script'")?;
    queue.send(McpAction::EvalTcl {
        script: script.to_string(),
    })
}

pub fn tool_workspace_roots(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let action = args.get("action").and_then(Value::as_str).ok_or("Missing 'action'")?;
    match action {
        "list" => queue.send(McpAction::ListRoots),
        "add" => {
            let path = args.get("path").and_then(Value::as_str).ok_or("Missing 'path'")?;
            queue.send(McpAction::AddRoot { path: path.to_string() })
        }
        "remove" => {
            let path = args.get("path").and_then(Value::as_str).ok_or("Missing 'path'")?;
            queue.send(McpAction::RemoveRoot { path: path.to_string() })
        }
        _ => Err(format!("Unknown workspace_roots action: {action}")),
    }
}
