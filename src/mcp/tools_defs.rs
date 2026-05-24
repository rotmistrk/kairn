//! MCP tool definitions — JSON schemas for tools/list response.

use serde_json::{json, Value};

/// Return the list of tool definitions for `tools/list`.
pub fn tool_definitions() -> Value {
    let mut tools = read_tool_definitions();
    tools.extend(super::tools_defs_write::write_tool_definitions());
    tools.extend(super::tools_defs_extra::extra_tool_definitions());
    Value::Array(tools)
}

/// Read-only tool definitions.
fn read_tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "list_tabs",
            "description": "List all open tabs with type, focus, modified, cursor, order",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "list_terminals",
            "description": "List terminal tabs with name, type, and index",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "get_terminal_content",
            "description": "Get terminal content by name or index",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Terminal tab name"},
                    "index": {"type": "integer", "description": "Terminal index (fallback)"}
                }
            }
        }),
        json!({
            "name": "get_todo_tree",
            "description": "Get the full todo tree (titles, checked state, nesting)",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "get_messages",
            "description": "Get the Messages log (errors, warnings, info from LSP, build, git, etc.)",
            "inputSchema": {"type": "object", "properties": {}}
        }),
        json!({
            "name": "get_tab_content",
            "description": "Get text content of a center-panel tab (editor buffer or results list) by name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (as shown in list_tabs)"}
                },
                "required": ["name"]
            }
        }),
        json!({
            "name": "update_todo",
            "description": "Modify the todo tree: toggle, add, remove, move, promote, demote, get/set notes",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["toggle", "add", "remove", "move_up", "move_down", "promote", "demote", "get_note", "set_note"],
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
                    },
                    "note": {
                        "type": "string",
                        "description": "Note content (required for 'set_note' action)"
                    }
                },
                "required": ["action", "path"]
            }
        }),
        json!({
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
        }),
    ]
}
