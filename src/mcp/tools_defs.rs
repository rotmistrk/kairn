//! MCP tool definitions — JSON schemas for tools/list response.

use serde_json::{json, Value};

/// Return the list of tool definitions for `tools/list`.
pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_tabs",
            "description": "List all open tabs with type, focus, modified, cursor, order",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "list_terminals",
            "description": "List terminal tabs with name, type, and index",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "get_terminal_content",
            "description": "Get terminal content by name or index",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Terminal tab name"},
                    "index": {"type": "integer", "description": "Terminal index (fallback)"}
                }
            }
        },
        {
            "name": "get_todo_tree",
            "description": "Get the full todo tree (titles, checked state, nesting)",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "get_messages",
            "description": "Get the Messages log (errors, warnings, info from LSP, build, git, etc.)",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "get_tab_content",
            "description": "Get text content of a center-panel tab (editor buffer or results list) by name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (as shown in list_tabs)"}
                },
                "required": ["name"]
            }
        },
        {
            "name": "update_todo",
            "description": "Modify the todo tree: toggle, add, remove, move, promote, demote items",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["toggle", "add", "remove", "move_up", "move_down", "promote", "demote"],
                        "description": "Action to perform"
                    },
                    "path": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Index path to the item"
                    },
                    "title": {
                        "type": "string",
                        "description": "Title for new item (required for 'add' action)"
                    }
                },
                "required": ["action", "path"]
            }
        },
        {
            "name": "add_subtree",
            "description": "Add a subtree of todo items as children of the item at path",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "Index path to the parent item"
                    },
                    "items": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "title": {"type": "string"},
                                "items": {"type": "array", "description": "Nested children (recursive)"}
                            },
                            "required": ["title"]
                        },
                        "description": "Tree of items to add"
                    }
                },
                "required": ["path", "items"]
            }
        },
        {
            "name": "open_file",
            "description": "Open an existing file in the editor",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "Relative path from project root"}
                },
                "required": ["path"]
            }
        },
        {
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
        },
        {
            "name": "close_tab",
            "description": "Close an editor tab by name/path",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (as shown in list_tabs)"}
                },
                "required": ["name"]
            }
        },
        {
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
        },
        {
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
        },
        {
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
        },
        {
            "name": "save_file",
            "description": "Save an open buffer to disk",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name"}
                },
                "required": ["name"]
            }
        },
        {
            "name": "get_diagnostics",
            "description": "Get LSP diagnostics (errors/warnings) for an open file",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (omit for all files)"}
                }
            }
        },
        {
            "name": "get_build_errors",
            "description": "Get parsed errors from the last build/test run",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "search_project",
            "description": "Search project files for a regex pattern (respects .gitignore)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string", "description": "Regex pattern to search for"}
                },
                "required": ["pattern"]
            }
        },
        {
            "name": "run_build",
            "description": "Run a build/test command (returns immediately, poll get_build_errors for results)",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "command": {"type": "string", "description": "Shell command (default: auto-detected build)"}
                }
            }
        },
        {
            "name": "split",
            "description": "Manipulate editor split panes: create, close, focus, or open file in other pane",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["vsplit", "hsplit", "close", "focus", "open"],
                        "description": "Split action to perform"
                    },
                    "file": {
                        "type": "string",
                        "description": "File path (for vsplit/hsplit/open)"
                    }
                },
                "required": ["action"]
            }
        }
    ])
}
