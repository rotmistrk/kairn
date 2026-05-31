//! MCP todo tool handlers.

use std::env;
use std::fs;

use duir_core::tree_ops::find_node_path;
use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};

pub fn tool_get_todo_tree() -> Result<Value, String> {
    let path = env::current_dir()
        .map(|d| d.join(".kairn.todo"))
        .map_err(|e| e.to_string())?;
    if !path.exists() {
        return Ok(json!({"items": []}));
    }
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let file: duir_core::TodoFile = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let items = serialize_todo_items(&file.items);
    Ok(json!({"title": file.title, "items": items}))
}

fn serialize_todo_items(items: &[duir_core::TodoItem]) -> Vec<Value> {
    items
        .iter()
        .map(|item| {
            let mut obj = json!({
                "id": item.id.0,
                "title": item.title,
                "completed": format!("{:?}", item.completed).to_lowercase(),
            });
            if item.important {
                obj["important"] = json!(true);
            }
            if !item.note.is_empty() {
                obj["note"] = json!(item.note);
            }
            if !item.items.is_empty() {
                obj["items"] = json!(serialize_todo_items(&item.items));
            }
            obj
        })
        .collect()
}

/// Resolve a todo item path from args: accepts either `"id"` (string NodeId) or `"path"` (ordinal array).
/// Prefers `id` when both are present.
fn resolve_path(args: &Map<String, Value>) -> Result<Vec<usize>, String> {
    if let Some(id_str) = args.get("id").and_then(Value::as_str) {
        let file_path = env::current_dir()
            .map(|d| d.join(".kairn.todo"))
            .map_err(|e| e.to_string())?;
        let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
        let file: duir_core::TodoFile = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        let node_id = duir_core::NodeId(id_str.to_string());
        find_node_path(&file, &node_id).ok_or_else(|| format!("Node not found: {id_str}"))
    } else if let Some(arr) = args.get("path").and_then(Value::as_array) {
        Ok(arr.iter().filter_map(Value::as_u64).map(|n| n as usize).collect())
    } else {
        Err("Missing 'id' or 'path'".to_string())
    }
}

pub fn tool_update_todo(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("Write operations disabled")?;
    let action_str = args.get("action").and_then(Value::as_str).ok_or("Missing 'action'")?;
    let path = resolve_path(args)?;

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
        "get_note" => {
            return tool_get_note(&path);
        }
        "set_note" => {
            let note = args.get("note").and_then(Value::as_str).unwrap_or("").to_string();
            McpAction::TodoSetNote { path, note }
        }
        _ => return Err(format!("Unknown action: {action_str}")),
    };

    queue.send(action)
}

fn tool_get_note(path: &[usize]) -> Result<Value, String> {
    let file_path = env::current_dir()
        .map(|d| d.join(".kairn.todo"))
        .map_err(|e| e.to_string())?;
    if !file_path.exists() {
        return Err("No todo file".to_string());
    }
    let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let file: duir_core::TodoFile = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    let mut items = &file.items[..];
    let mut item = None;
    for &idx in path {
        let it = items.get(idx).ok_or("Item not found")?;
        item = Some(it);
        items = &it.items;
    }
    let note = item.map(|i| i.note.as_str()).unwrap_or("");
    Ok(json!({"note": note}))
}

pub fn tool_add_subtree(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("Write operations disabled")?;
    let path = resolve_path(args)?;
    let items = args
        .get("items")
        .and_then(Value::as_array)
        .ok_or("Missing 'items'")?
        .clone();

    queue.send(McpAction::TodoAddSubtree { path, items })
}
