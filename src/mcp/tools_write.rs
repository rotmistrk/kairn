//! MCP write tools — file/tab management (Tier 2).

use serde_json::{json, Map, Value};

use super::args::{opt_bool, opt_str, require_queue, require_str, require_u64};
use super::commands::{McpAction, McpCommandQueue};

pub fn tool_open_file(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let path = require_str(args, "path")?;
    queue.send(McpAction::OpenFile { path: path.to_string() })?;
    Ok(json!({"opened": path}))
}

pub fn tool_highlight_code(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let path = require_str(args, "path")?;
    let ranges_arr = args.get("ranges").and_then(Value::as_array).ok_or("Missing 'ranges'")?;
    let ranges: Vec<(u32, u32)> = ranges_arr
        .iter()
        .filter_map(|r| {
            let start = r.get("start_line").and_then(Value::as_u64)? as u32;
            let end = r.get("end_line").and_then(Value::as_u64).unwrap_or(start as u64) as u32;
            Some((start, end))
        })
        .collect();
    if ranges.is_empty() {
        return Err("No valid ranges provided".to_string());
    }
    queue.send(McpAction::HighlightCode {
        path: path.to_string(),
        ranges,
    })?;
    Ok(json!({"highlighted": path}))
}

pub fn tool_create_file(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let path = require_str(args, "path")?;
    let content = opt_str(args, "content", "");
    queue.send(McpAction::CreateFile {
        path: path.to_string(),
        content: content.to_string(),
    })?;
    Ok(json!({"created": path}))
}

pub fn tool_close_tab(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    queue.send(McpAction::CloseTab { name: name.to_string() })?;
    Ok(json!({"closed": name}))
}

pub fn tool_edit_buffer(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    let start = require_u64(args, "start_line")? as usize;
    let end = require_u64(args, "end_line")? as usize;
    let text = require_str(args, "text")?;
    queue.send(McpAction::EditBuffer {
        name: name.to_string(),
        start_line: start,
        end_line: end,
        text: text.to_string(),
    })
}

pub fn tool_insert_text(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    let line = require_u64(args, "line")? as usize;
    let col = require_u64(args, "col")? as usize;
    let text = require_str(args, "text")?;
    queue.send(McpAction::InsertText {
        name: name.to_string(),
        line,
        col,
        text: text.to_string(),
    })
}

pub fn tool_set_cursor(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    let line = require_u64(args, "line")? as usize;
    let col = require_u64(args, "col")? as usize;
    queue.send(McpAction::SetCursor {
        name: name.to_string(),
        line,
        col,
    })
}

pub fn tool_save_file(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    queue.send(McpAction::SaveFile { name: name.to_string() })
}

pub fn tool_get_diagnostics(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = opt_str(args, "name", "");
    queue.send(McpAction::GetDiagnostics { name: name.to_string() })
}

pub fn tool_get_build_errors(cmd_queue: Option<&McpCommandQueue>, _args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    queue.send(McpAction::GetBuildErrors)
}

pub fn tool_search_project(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let pattern = require_str(args, "pattern")?;
    let all_roots = opt_bool(args, "all_roots", false);
    queue.send(McpAction::SearchProject {
        pattern: pattern.to_string(),
        all_roots,
    })
}

pub fn tool_run_build(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let command = opt_str(args, "command", "");
    queue.send(McpAction::RunBuild {
        command: command.to_string(),
    })
}

pub fn tool_diff_revert(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let name = require_str(args, "name")?;
    queue.send(McpAction::DiffRevert { name: name.to_string() })
}

pub fn tool_lsp_control(cmd_queue: Option<&McpCommandQueue>, args: &Map<String, Value>) -> Result<Value, String> {
    let queue = require_queue(cmd_queue)?;
    let action = require_str(args, "action")?;
    let lang = opt_str(args, "lang", "*");
    let value = opt_str(args, "value", "");
    let command = format!("{action} {lang} {value}").trim().to_string();
    queue.send(McpAction::LspControl { command })
}
