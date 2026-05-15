//! MCP tool definitions and dispatch.

use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

use super::commands::McpCommandQueue;
use super::snapshot::McpSnapshot;

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
                        "description": "Index path to the item (e.g. [0] for first root, [2,1] for second child of third root)"
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
                        "description": "Tree of items to add (each with title and optional nested items)"
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
        }
    ])
}

/// Dispatch a tool call to the appropriate handler.
pub fn handle_tool_call(
    snapshot: &Arc<Mutex<McpSnapshot>>,
    cmd_queue: Option<&McpCommandQueue>,
    name: &str,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    match name {
        "list_tabs" => tool_list_tabs(snapshot),
        "list_terminals" => tool_list_terminals(snapshot),
        "get_terminal_content" => tool_get_terminal_content(snapshot, args),
        "get_messages" => tool_get_messages(snapshot),
        "get_tab_content" => tool_get_tab_content(snapshot, args),
        "get_todo_tree" => super::tools_todo::tool_get_todo_tree(),
        "update_todo" => super::tools_todo::tool_update_todo(cmd_queue, args),
        "add_subtree" => super::tools_todo::tool_add_subtree(cmd_queue, args),
        "open_file" => super::tools_write::tool_open_file(cmd_queue, args),
        "create_file" => super::tools_write::tool_create_file(cmd_queue, args),
        "close_tab" => super::tools_write::tool_close_tab(cmd_queue, args),
        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn tool_list_tabs(snapshot: &Arc<Mutex<McpSnapshot>>) -> Result<Value, String> {
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    let tabs: Vec<Value> = snap
        .tabs
        .iter()
        .map(|t| {
            let mut obj = json!({
                "name": t.name,
                "type": t.tab_type,
                "focused": t.focused,
                "active": t.active,
                "modified": t.modified,
                "order": t.order,
            });
            if let Some(ref path) = t.path {
                obj["path"] = json!(path);
            }
            if let Some(ref c) = t.cursor {
                obj["cursor"] = json!({"line": c.line, "col": c.col});
            }
            if let Some(ref s) = t.selection {
                obj["selection"] = json!({
                    "start": {"line": s.start_line, "col": s.start_col},
                    "end": {"line": s.end_line, "col": s.end_col},
                });
            }
            obj
        })
        .collect();
    Ok(json!({"focused_slot": snap.focused_slot, "tabs": tabs}))
}

fn tool_list_terminals(snapshot: &Arc<Mutex<McpSnapshot>>) -> Result<Value, String> {
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    let terms: Vec<Value> = snap
        .terminals
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "type": t.terminal_type,
                "index": t.index,
            })
        })
        .collect();
    Ok(json!(terms))
}

fn tool_get_terminal_content(snapshot: &Arc<Mutex<McpSnapshot>>, args: &Map<String, Value>) -> Result<Value, String> {
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    let name = args.get("name").and_then(Value::as_str);
    let index = args.get("index").and_then(Value::as_u64).map(|n| n as usize);

    let term = if let Some(name) = name {
        snap.terminals.iter().find(|t| t.name == name)
    } else if let Some(idx) = index {
        snap.terminals.iter().find(|t| t.index == idx)
    } else {
        return Err("Provide 'name' or 'index' argument".to_owned());
    };

    match term {
        Some(t) => Ok(json!({"name": t.name, "content": t.content})),
        None => Err("Terminal not found".to_owned()),
    }
}

fn tool_get_messages(snapshot: &Arc<Mutex<McpSnapshot>>) -> Result<Value, String> {
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    Ok(json!(snap.messages.join("\n")))
}

fn tool_get_tab_content(snapshot: &Arc<Mutex<McpSnapshot>>, args: &Map<String, Value>) -> Result<Value, String> {
    let name = args
        .get("name")
        .and_then(Value::as_str)
        .ok_or("Missing 'name' argument")?;
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    match snap.tab_contents.get(name) {
        Some(content) => Ok(json!({"name": name, "content": content})),
        None => Err(format!("Tab not found or has no readable content: {name}")),
    }
}
