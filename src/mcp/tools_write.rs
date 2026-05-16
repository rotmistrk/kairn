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

pub fn tool_edit_buffer(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).ok_or("Missing 'name'")?;
    let start = args
        .get("start_line")
        .and_then(Value::as_u64)
        .ok_or("Missing 'start_line'")? as usize;
    let end = args
        .get("end_line")
        .and_then(Value::as_u64)
        .ok_or("Missing 'end_line'")? as usize;
    let text = args.get("text").and_then(Value::as_str).ok_or("Missing 'text'")?;
    queue.send(McpAction::EditBuffer {
        name: name.to_string(),
        start_line: start,
        end_line: end,
        text: text.to_string(),
    })
}

pub fn tool_insert_text(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).ok_or("Missing 'name'")?;
    let line = args.get("line").and_then(Value::as_u64).ok_or("Missing 'line'")? as usize;
    let col = args.get("col").and_then(Value::as_u64).ok_or("Missing 'col'")? as usize;
    let text = args.get("text").and_then(Value::as_str).ok_or("Missing 'text'")?;
    queue.send(McpAction::InsertText {
        name: name.to_string(),
        line,
        col,
        text: text.to_string(),
    })
}

pub fn tool_set_cursor(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).ok_or("Missing 'name'")?;
    let line = args.get("line").and_then(Value::as_u64).ok_or("Missing 'line'")? as usize;
    let col = args.get("col").and_then(Value::as_u64).ok_or("Missing 'col'")? as usize;
    queue.send(McpAction::SetCursor {
        name: name.to_string(),
        line,
        col,
    })
}

pub fn tool_save_file(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).ok_or("Missing 'name'")?;
    queue.send(McpAction::SaveFile { name: name.to_string() })
}

pub fn tool_get_diagnostics(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).unwrap_or("");
    queue.send(McpAction::GetDiagnostics { name: name.to_string() })
}

pub fn tool_get_build_errors(cmd_queue: Option<&McpCommandQueue>, _args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    queue.send(McpAction::GetBuildErrors)
}

pub fn tool_search_project(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let pattern = args.get("pattern").and_then(Value::as_str).ok_or("Missing 'pattern'")?;
    queue.send(McpAction::SearchProject {
        pattern: pattern.to_string(),
    })
}

pub fn tool_run_build(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let command = args.get("command").and_then(Value::as_str).unwrap_or("");
    queue.send(McpAction::RunBuild {
        command: command.to_string(),
    })
}

pub fn tool_diff_revert(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let name = args.get("name").and_then(Value::as_str).ok_or("Missing 'name'")?;
    queue.send(McpAction::DiffRevert { name: name.to_string() })
}
