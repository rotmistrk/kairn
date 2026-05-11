//! MCP tool definitions and dispatch.

use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

use super::snapshot::McpSnapshot;

/// Return the list of tool definitions for `tools/list`.
pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_tabs",
            "description": "List all open tabs with type and title",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "list_terminals",
            "description": "List terminal tabs: name, status, type (shell/kiro)",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "get_terminal_content",
            "description": "Get terminal scrollback + visible content by tab name",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": {"type": "string", "description": "Terminal tab name"}
                },
                "required": ["name"]
            }
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
        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn tool_list_tabs(snapshot: &Arc<Mutex<McpSnapshot>>) -> Result<Value, String> {
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    let tabs: Vec<Value> = snap
        .tabs
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "type": t.tab_type,
                "path": t.path,
            })
        })
        .collect();
    Ok(json!(tabs))
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
            })
        })
        .collect();
    Ok(json!(terms))
}

fn tool_get_terminal_content(snapshot: &Arc<Mutex<McpSnapshot>>, args: &Map<String, Value>) -> Result<Value, String> {
    let name = args
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| "Missing required argument: name".to_owned())?;
    let snap = snapshot.lock().map_err(|e| e.to_string())?;
    let term = snap
        .terminals
        .iter()
        .find(|t| t.name == name)
        .ok_or_else(|| format!("Terminal not found: {name}"))?;
    Ok(json!(term.content))
}
