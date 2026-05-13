//! MCP tool definitions and dispatch.

use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

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
        }
    ])
}

/// Dispatch a tool call to the appropriate handler.
pub fn handle_tool_call(
    snapshot: &Arc<Mutex<McpSnapshot>>,
    name: &str,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    match name {
        "list_tabs" => tool_list_tabs(snapshot),
        "list_terminals" => tool_list_terminals(snapshot),
        "get_terminal_content" => tool_get_terminal_content(snapshot, args),
        "get_todo_tree" => tool_get_todo_tree(snapshot),
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
                "modified": t.modified,
                "order": t.order,
            });
            if let Some(ref path) = t.path {
                obj["path"] = json!(path);
            }
            if let Some(ref c) = t.cursor {
                obj["cursor"] = json!({"line": c.line, "col": c.col});
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
