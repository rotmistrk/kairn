//! MCP tool definitions and dispatch.

use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};

use super::commands::{McpAction, McpCommandQueue};
use super::snapshot::McpSnapshot;
use super::tools_extra::{tool_eval_tcl, tool_git_ops, tool_lsp_semantic, tool_send_terminal_input, tool_undo_redo};
use super::tools_todo::{tool_add_subtree, tool_get_todo_tree, tool_update_todo};
use super::tools_write::{
    tool_close_tab, tool_create_file, tool_diff_revert, tool_edit_buffer, tool_get_build_errors, tool_get_diagnostics,
    tool_insert_text, tool_lsp_control, tool_open_file, tool_run_build, tool_save_file, tool_search_project,
    tool_set_cursor,
};

pub use super::tools_defs::tool_definitions;

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
        "get_todo_tree" => tool_get_todo_tree(),
        "update_todo" => tool_update_todo(cmd_queue, args),
        "add_subtree" => tool_add_subtree(cmd_queue, args),
        "open_file" => tool_open_file(cmd_queue, args),
        "create_file" => tool_create_file(cmd_queue, args),
        "close_tab" => tool_close_tab(cmd_queue, args),
        "edit_buffer" => tool_edit_buffer(cmd_queue, args),
        "insert_text" => tool_insert_text(cmd_queue, args),
        "set_cursor" => tool_set_cursor(cmd_queue, args),
        "save_file" => tool_save_file(cmd_queue, args),
        "get_diagnostics" => tool_get_diagnostics(cmd_queue, args),
        "get_build_errors" => tool_get_build_errors(cmd_queue, args),
        "search_project" => tool_search_project(cmd_queue, args),
        "run_build" => tool_run_build(cmd_queue, args),
        "split" => tool_split(snapshot, cmd_queue, args),
        "diff_revert" => tool_diff_revert(cmd_queue, args),
        "lsp_control" => tool_lsp_control(cmd_queue, args),
        "send_terminal_input" => tool_send_terminal_input(cmd_queue, args),
        "git_ops" => tool_git_ops(cmd_queue, args),
        "lsp_semantic" => tool_lsp_semantic(cmd_queue, args),
        "undo_redo" => tool_undo_redo(cmd_queue, args),
        "eval_tcl" => tool_eval_tcl(cmd_queue, args),
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

fn tool_split(
    snapshot: &Arc<Mutex<McpSnapshot>>,
    cmd_queue: Option<&McpCommandQueue>,
    args: &Map<String, Value>,
) -> Result<Value, String> {
    let action = args
        .get("action")
        .and_then(Value::as_str)
        .ok_or("Missing 'action' argument")?;
    let file = args.get("file").and_then(Value::as_str).map(String::from);

    if action == "status" {
        let snap = snapshot.lock().map_err(|e| e.to_string())?;
        return Ok(serde_json::json!({
            "active": snap.split_direction != "none",
            "direction": snap.split_direction,
            "linked_scroll": snap.split_linked,
        }));
    }

    let queue = cmd_queue.ok_or("MCP command queue not available")?;
    let mcp_action = match action {
        "vsplit" => McpAction::SplitVertical { file },
        "hsplit" => McpAction::SplitHorizontal { file },
        "close" => McpAction::SplitClose,
        "focus" => McpAction::SplitFocus,
        "open" => {
            let path = file.ok_or("'file' required for 'open' action")?;
            McpAction::SplitOpen { path }
        }
        "linked" => {
            let on = args.get("value").and_then(Value::as_bool).unwrap_or(true);
            McpAction::SplitLinked { on }
        }
        other => return Err(format!("Unknown split action: {other}")),
    };
    queue.send(mcp_action)
}
