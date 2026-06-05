//! MCP tool definitions — JSON schemas for tools/list response.

use serde_json::{json, Value};

use super::tools_defs_extra::extra_tool_definitions;
use super::tools_defs_write::write_tool_definitions;

/// Return the list of tool definitions for `tools/list`.
pub fn tool_definitions() -> Value {
    let mut tools = read_tool_definitions();
    tools.extend(write_tool_definitions());
    tools.extend(extra_tool_definitions());
    Value::Array(tools)
}

/// Read-only tool definitions.
fn read_tool_definitions() -> Vec<Value> {
    let mut tools = list_tab_definitions();
    tools.extend(content_definitions());
    tools.extend(update_todo_definition());
    tools.extend(add_subtree_definition());
    tools
}

fn list_tab_definitions() -> Vec<Value> {
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
    ]
}

fn content_definitions() -> Vec<Value> {
    vec![
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
            "description": "Get text content of a center-panel tab (editor buffer or results) by name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Tab name (as shown in list_tabs)"}
                },
                "required": ["name"]
            }
        }),
    ]
}

fn update_todo_definition() -> Vec<Value> {
    vec![json!({
        "name": "update_todo",
        "description": "Modify the todo tree: toggle, add, remove, move, promote, demote, \
            get/set notes, set_priority, set_completed. Identify items by 'id' (stable UUID) \
            or 'path' (ordinal index array).",
        "inputSchema": update_todo_schema()
    })]
}

fn update_todo_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "action": {
                "type": "string",
                "enum": ["toggle", "add", "remove", "move_up", "move_down", "promote",
                         "demote", "get_note", "set_note", "set_priority", "set_completed"],
                "description": "Action to perform"
            },
            "id": {"type": "string", "description": "Stable UUID of the item (preferred over path)"},
            "path": {"type": "array", "items": {"type": "integer"},
                     "description": "Index path to the item (fallback if id not provided)"},
            "title": {"type": "string", "description": "Title for new item (required for 'add')"},
            "note": {"type": "string", "description": "Note content (required for 'set_note')"},
            "priority": {"type": "integer", "description": "Priority 0-5 (for 'set_priority'; 0=none, 5=highest)"},
            "state": {"type": "string", "enum": ["open", "done", "partial"],
                      "description": "Completion state (for 'set_completed')"}
        },
        "required": ["action"]
    })
}

fn add_subtree_definition() -> Vec<Value> {
    vec![json!({
        "name": "add_subtree",
        "description": "Add a subtree of todo items as children of the item at path. \
            Identify parent by 'id' (stable UUID) or 'path' (ordinal index array).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "id": {
                    "type": "string",
                    "description": "Stable UUID of the parent item (preferred over path)"
                },
                "path": {
                    "type": "array",
                    "items": {"type": "integer"},
                    "description": "Index path to the parent item (fallback if id not provided)"
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
            "required": ["items"]
        }
    })]
}
