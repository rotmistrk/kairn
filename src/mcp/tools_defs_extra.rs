//! MCP tool definitions — terminal, git, LSP, undo/redo, eval.

use serde_json::{json, Value};

/// Additional tool definitions for `tools/list`.
pub fn extra_tool_definitions() -> Vec<Value> {
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
        json!({
            "name": "lsp_semantic",
            "description": "LSP semantic queries: hover, definition, references, rename, code-action",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {"type": "string", "enum": ["hover", "definition", "references", "rename", "code-action"]},
                    "name": {"type": "string", "description": "Tab name (default: active editor)"},
                    "new_name": {"type": "string", "description": "New name (for rename action)"}
                },
                "required": ["action"]
            }
        }),
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
