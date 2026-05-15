//! MCP write tools — file/tab management (Tier 2).

use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};

pub fn tool_open_file(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or("Missing 'path' argument")?;
    queue.send(McpAction::OpenFile { path: path.to_string() })?;
    Ok(json!({"opened": path}))
}

pub fn tool_create_file(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or("Missing 'path' argument")?;
    let content = args.get("content").and_then(Value::as_str).unwrap_or("");
    queue.send(McpAction::CreateFile {
        path: path.to_string(),
        content: content.to_string(),
    })?;
    Ok(json!({"created": path}))
}

pub fn tool_close_tab(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args
        .get("name")
        .and_then(Value::as_str)
        .ok_or("Missing 'name' argument")?;
    queue.send(McpAction::CloseTab { name: name.to_string() })?;
    Ok(json!({"closed": name}))
}
