//! MCP tool definitions — terminal, git, LSP, undo/redo, eval.

use serde_json::{json, Value};

/// Additional tool definitions for `tools/list`.
pub fn extra_tool_definitions() -> Vec<Value> {
    let mut tools = terminal_and_git_definitions();
    tools.extend(lsp_and_eval_definitions());
    tools.push(workspace_roots_definition());
    tools.extend(clipboard_definitions());
    tools
}

fn terminal_and_git_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "send_terminal_input",
            "description": "Send input text to a terminal/shell tab (simulates typing)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Terminal tab name"},
                    "input": {"type": "string", "description": "Text to send (use \\n for Enter)"}
                },
                "required": ["name", "input"]
            }
        }),
        json!({
            "name": "git_ops",
            "description": "Git operations: stage, unstage, or commit",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["stage", "unstage", "commit"]},
                    "file": {"type": "string", "description": "File path (for stage/unstage)"},
                    "message": {"type": "string", "description": "Commit message (for commit)"}
                },
                "required": ["action"]
            }
        }),
    ]
}

fn lsp_and_eval_definitions() -> Vec<Value> {
    let mut tools = vec![json!({
        "name": "lsp_semantic",
        "description": "LSP semantic queries: hover, definition, references, rename, code-action",
        "inputSchema": {
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["hover", "definition", "references", "rename", "code-action"]
                },
                "name": {"type": "string", "description": "Tab name (default: active editor)"},
                "new_name": {"type": "string", "description": "New name (for rename action)"}
            },
            "required": ["action"]
        }
    })];
    tools.extend(undo_and_eval_definitions());
    tools
}

fn undo_and_eval_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "undo_redo",
            "description": "Undo or redo in an editor buffer",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["undo", "redo"]},
                    "name": {"type": "string", "description": "Tab name (default: active editor)"}
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "eval_tcl",
            "description": "Evaluate a Tcl script in the kairn scripting engine",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "script": {"type": "string", "description": "Tcl script to evaluate"}
                },
                "required": ["script"]
            }
        }),
    ]
}

fn workspace_roots_definition() -> Value {
    json!({
        "name": "workspace_roots",
        "description": "Manage workspace roots: list, add, or remove project directories",
        "inputSchema": {
            "type": "object",
            "properties": {
                "action": {"type": "string", "enum": ["list", "add", "remove"]},
                "path": {"type": "string", "description": "Directory path (for add/remove)"}
            },
            "required": ["action"]
        }
    })
}

fn clipboard_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "clipboard_copy",
            "description": "Copy text to the clipboard ring",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "text": {"type": "string", "description": "Text to copy"},
                    "source": {"type": "string", "description": "Source label (default: mcp)"}
                },
                "required": ["text"]
            }
        }),
        json!({
            "name": "clipboard_paste",
            "description": "Read the top of the clipboard ring (tries system clipboard first)",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "clipboard_list",
            "description": "List all clipboard ring entries (first line, line count, source)",
            "inputSchema": {"type": "object", "properties": {}}
        }),
    ]
}
