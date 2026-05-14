//! MCP tool definitions and dispatch.

use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};
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
        "get_todo_tree" => tool_get_todo_tree(snapshot),
        "update_todo" => tool_update_todo(cmd_queue, args),
        "add_subtree" => tool_add_subtree(cmd_queue, args),
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

fn tool_get_todo_tree(_snapshot: &Arc<Mutex<McpSnapshot>>) -> Result<Value, String> {
    let path = std::env::current_dir()
        .map(|d| d.join(".kairn.todo"))
        .map_err(|e| e.to_string())?;
    if !path.exists() {
        return Ok(json!({"items": []}));
    }
    let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file: duir_core::TodoFile = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let items = serialize_todo_items(&file.items);
    Ok(json!({"title": file.title, "items": items}))
}

fn serialize_todo_items(items: &[duir_core::TodoItem]) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            let mut obj = json!({
                "title": item.title,
                "completed": format!("{:?}", item.completed).to_lowercase(),
            });
            if item.important {
                obj["important"] = json!(true);
            }
            if !item.items.is_empty() {
                obj["items"] = json!(serialize_todo_items(&item.items));
            }
            obj
        })
        .collect()
}

fn tool_update_todo(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("Write operations disabled")?;
    let action_str = args.get("action").and_then(Value::as_str).ok_or("Missing 'action'")?;
    let path: Vec<usize> = args
        .get("path")
        .and_then(Value::as_array)
        .ok_or("Missing 'path'")?
        .iter()
        .filter_map(Value::as_u64)
        .map(|n| n as usize)
        .collect();

    let action = match action_str {
        "toggle" => McpAction::TodoToggle { path },
        "add" => {
            let title = args
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("New task")
                .to_string();
            McpAction::TodoAdd { path, title }
        }
        "remove" => McpAction::TodoRemove { path },
        "move_up" => McpAction::TodoMoveUp { path },
        "move_down" => McpAction::TodoMoveDown { path },
        "promote" => McpAction::TodoPromote { path },
        "demote" => McpAction::TodoDemote { path },
        _ => return Err(format!("Unknown action: {action_str}")),
    };

    queue.send(action)
}

fn tool_add_subtree(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("Write operations disabled")?;
    let path: Vec<usize> = args
        .get("path")
        .and_then(Value::as_array)
        .ok_or("Missing 'path'")?
        .iter()
        .filter_map(Value::as_u64)
        .map(|n| n as usize)
        .collect();
    let items = args
        .get("items")
        .and_then(Value::as_array)
        .ok_or("Missing 'items'")?
        .clone();

    queue.send(McpAction::TodoAddSubtree { path, items })
}
