//! MCP tool handlers — terminal, git, LSP, undo/redo, eval.

use serde_json::{json, Map, Value};

use super::args::{opt_str, require_queue, require_str};
use super::commands::{McpAction, McpCommandQueue};

pub fn tool_send_terminal_input(
    cmd_queue: Option<&McpCommandQueue>,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    let input = require_str(args, "input")?;
    queue.send(McpAction::SendTerminalInput {
        name: name.to_string(),
        input: input.to_string(),
    })?;
    Ok(json!({"sent": true, "target": name}))
}

pub fn tool_git_ops(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let action = require_str(args, "action")?;
    match action {
        "stage" => {
            let file = require_str(args, "file")?;
            queue.send(McpAction::GitStage { file: file.to_string() })
        }
        "unstage" => {
            let file = require_str(args, "file")?;
            queue.send(McpAction::GitUnstage { file: file.to_string() })
        }
        "commit" => {
            let message = require_str(args, "message")?;
            queue.send(McpAction::GitCommit {
                message: message.to_string(),
            })
        }
        _ => Err(format!("Unknown git action: {action}")),
    }
}

pub fn tool_lsp_semantic(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let action = require_str(args, "action")?;
    let name = opt_str(args, "name", "").to_string();
    match action {
        "hover" => queue.send(McpAction::LspHover { name }),
        "definition" => queue.send(McpAction::LspDefinition { name }),
        "references" => queue.send(McpAction::LspReferences { name }),
        "rename" => {
            let new_name = require_str(args, "new_name")?;
            queue.send(McpAction::LspRename {
                name,
                new_name: new_name.to_string(),
            })
        }
        "code-action" => queue.send(McpAction::LspCodeAction { name }),
        "format" => queue.send(McpAction::LspFormat { name }),
        _ => Err(format!("Unknown LSP action: {action}")),
    }
}

pub fn tool_undo_redo(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let action = require_str(args, "action")?;
    let name = opt_str(args, "name", "").to_string();
    match action {
        "undo" => queue.send(McpAction::Undo { name }),
        "redo" => queue.send(McpAction::Redo { name }),
        _ => Err(format!("Unknown action: {action}")),
    }
}

pub fn tool_eval_tcl(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let script = require_str(args, "script")?;
    queue.send(McpAction::EvalTcl {
        script: script.to_string(),
    })
}

pub fn tool_workspace_roots(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let action = require_str(args, "action")?;
    match action {
        "list" => queue.send(McpAction::ListRoots),
        "add" => {
            let path = require_str(args, "path")?;
            queue.send(McpAction::AddRoot { path: path.to_string() })
        }
        "remove" => {
            let path = require_str(args, "path")?;
            queue.send(McpAction::RemoveRoot { path: path.to_string() })
        }
        _ => Err(format!("Unknown workspace_roots action: {action}")),
    }
}
