//! MCP todo tool handlers.

use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};

pub fn tool_get_todo_tree() -> Result<Value, String> {
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

pub fn tool_update_todo(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
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

pub fn tool_add_subtree(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
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
