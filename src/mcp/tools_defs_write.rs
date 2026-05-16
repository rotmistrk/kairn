//! MCP write-tool definitions — schemas for mutation tools.

use serde_json::{json, Value};

/// Write/mutation tool definitions for `tools/list`.
pub fn write_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "open_file",
            "description": "Open an existing file in the editor",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Relative path from project root"}
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "create_file",
            "description": "Create a new file on disk and open it in the editor",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Relative path from project root"},
                    "content": {"type": "string", "description": "Initial file content (default: empty)"}
                },
                "required": ["path"]
            }
        }),
        json!({
            "name": "close_tab",
            "description": "Close an editor tab by name/path",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (as shown in list_tabs)"}
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "edit_buffer",
            "description": "Replace a line range in an open buffer (0-indexed, end exclusive)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name"},
                    "start_line": {"type": "integer", "description": "First line to replace (0-indexed)"},
                    "end_line": {"type": "integer", "description": "Line after last to replace (exclusive)"},
                    "text": {"type": "string", "description": "Replacement text"}
                },
                "required": ["name", "start_line", "end_line", "text"]
            }
        }),
        json!({
            "name": "insert_text",
            "description": "Insert text at a position in an open buffer",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name"},
                    "line": {"type": "integer", "description": "Line number (0-indexed)"},
                    "col": {"type": "integer", "description": "Column (0-indexed)"},
                    "text": {"type": "string", "description": "Text to insert"}
                },
                "required": ["name", "line", "col", "text"]
            }
        }),
        json!({
            "name": "set_cursor",
            "description": "Move cursor to line:col in a tab",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name"},
                    "line": {"type": "integer", "description": "Line (0-indexed)"},
                    "col": {"type": "integer", "description": "Column (0-indexed)"}
                },
                "required": ["name", "line", "col"]
            }
        }),
        json!({
            "name": "save_file",
            "description": "Save an open buffer to disk",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name"}
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "get_diagnostics",
            "description": "Get LSP diagnostics (errors/warnings) for an open file",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (omit for all files)"}
                }
            }
        }),
        json!({
            "name": "get_build_errors",
            "description": "Get parsed errors from the last build/test run",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "search_project",
            "description": "Search project files for a regex pattern (respects .gitignore)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search for"}
                },
                "required": ["pattern"]
            }
        }),
        json!({
            "name": "run_build",
            "description": "Run a build/test command (returns immediately, poll get_build_errors for results)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command (default: auto-detected build)"}
                }
            }
        }),
        json!({
            "name": "split",
            "description": "Manipulate editor split panes: create, close, focus, status, or linked scroll",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["vsplit", "hsplit", "close", "focus", "open", "status", "linked"],
                        "description": "Split action to perform"
                    },
                    "file": {
                        "type": "string",
                        "description": "File path (for vsplit/hsplit/open)"
                    },
                    "value": {
                        "type": "boolean",
                        "description": "Value for linked action (true/false)"
                    }
                },
                "required": ["action"]
            }
        }),
        json!({
            "name": "diff_revert",
            "description": "Revert the diff hunk under cursor (requires diff mode active)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name"}
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "lsp_control",
            "description": "Control LSP servers: start, restart, stop, set timeout, configure args",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["start", "restart", "stop", "timeout", "args", "status"],
                        "description": "Action to perform"
                    },
                    "lang": {
                        "type": "string",
                        "description": "Language glob pattern (e.g. 'rust', 'type*', '*')"
                    },
                    "value": {
                        "type": "string",
                        "description": "Value for timeout (seconds) or args (command + args)"
                    }
                },
                "required": ["action"]
            }
        }),
    ]
}
