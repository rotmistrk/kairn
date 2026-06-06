//! MCP command drain — dispatches MCP write requests to app state.

use std::path::PathBuf;

use txv_core::program::CommandContext;

use crate::commands::{
    OpenFileRequest, CM_CODE_ACTION, CM_GIT_COMMIT, CM_GIT_STAGE, CM_GIT_UNSTAGE, CM_LSP_FIND_REFS, CM_LSP_FORMAT,
    CM_LSP_GOTO_DEF, CM_LSP_HOVER, CM_LSP_RENAME, CM_OPEN_IN_SPLIT, CM_SPLIT_CLOSE, CM_SPLIT_FOCUS,
    CM_SPLIT_LINKED,
};
use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::handler_lsp_cmd::handle_lsp_command as handle_lsp_cmd;
use crate::handler_mcp_build::{mcp_get_build_errors, mcp_run_build, mcp_search_project};
use crate::handler_mcp_clipboard::*;
use crate::handler_mcp_edit::{mcp_edit_buffer, mcp_get_diagnostics, mcp_insert_text, mcp_save_file, mcp_set_cursor};
use crate::handler_mcp_helpers::*;
use crate::mcp::commands::McpAction;
use crate::views::todo_tree::TodoTreeView;

/// Drain MCP write commands and execute them on the live app state.
pub fn drain_mcp(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(ref queue) = state.mcp.commands else {
        return;
    };
    let requests = queue.drain();
    if requests.is_empty() {
        return;
    }
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        for req in requests {
            let _ = req.reply.send(Err("Desktop unavailable".to_string()));
        }
        return;
    };
    for req in requests {
        let result = dispatch_mcp_action(&req.action, desktop, state, ctx.sink);
        let _ = req.reply.send(result);
    }
}

fn dispatch_mcp_action(
    action: &McpAction,
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    sink: &txv_core::prelude::EventSink,
) -> Result<serde_json::Value, String> {
    match action {
        McpAction::OpenFile { path } => mcp_open_file(desktop, state, sink, path),
        McpAction::HighlightCode { path, ranges } => mcp_highlight_code(desktop, state, sink, path, ranges),
        McpAction::CreateFile { path, content } => mcp_create_file(desktop, state, sink, path, content),
        McpAction::CloseTab { name } => mcp_close_tab(desktop, state, name),
        McpAction::ClipboardCopy { text, source } => mcp_clipboard_copy(state, text, source),
        McpAction::ClipboardPaste => mcp_clipboard_paste(state),
        McpAction::ClipboardList => mcp_clipboard_list(state),
        McpAction::EditBuffer {
            name,
            start_line,
            end_line,
            text,
        } => mcp_edit_buffer(desktop, name, *start_line, *end_line, text),
        McpAction::InsertText { name, line, col, text } => mcp_insert_text(desktop, name, *line, *col, text),
        McpAction::SetCursor { name, line, col } => mcp_set_cursor(desktop, name, *line, *col),
        McpAction::SaveFile { name } => mcp_save_file(desktop, name),
        McpAction::GetDiagnostics { name } => mcp_get_diagnostics(desktop, name),
        McpAction::GetBuildErrors => mcp_get_build_errors(state),
        McpAction::SearchProject { pattern, all_roots } => mcp_search_project(state, pattern, *all_roots),
        McpAction::RunBuild { command } => mcp_run_build(state, sink, command),
        McpAction::DiffRevert { name } => mcp_diff_revert(desktop, name),
        McpAction::SendTerminalInput { name, input } => mcp_send_terminal_input(desktop, name, input),
        McpAction::Undo { name } => mcp_undo_redo(desktop, name, true),
        McpAction::Redo { name } => mcp_undo_redo(desktop, name, false),
        McpAction::EvalTcl { script } => mcp_eval_tcl(state, script),
        McpAction::ListRoots => mcp_list_roots(state),
        McpAction::AddRoot { path } => mcp_add_root(state, sink, path),
        McpAction::RemoveRoot { path } => mcp_remove_root(state, sink, path),
        _ => dispatch_mcp_split_git_lsp(action, desktop, state, sink),
    }
}

fn dispatch_mcp_split_git_lsp(
    action: &McpAction,
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
    state: &mut AppState,
    sink: &txv_core::prelude::EventSink,
) -> Result<serde_json::Value, String> {
    match action {
        McpAction::SplitVertical { .. }
        | McpAction::SplitHorizontal { .. }
        | McpAction::SplitClose
        | McpAction::SplitFocus
        | McpAction::SplitOpen { .. }
        | McpAction::SplitLinked { .. } => dispatch_mcp_split(action, sink),
        McpAction::LspControl { command } => {
            let msg = handle_lsp_cmd(command, state);
            Ok(serde_json::json!({"result": msg}))
        }
        McpAction::GitStage { .. } | McpAction::GitUnstage { .. } | McpAction::GitCommit { .. } => {
            dispatch_mcp_git(action, sink)
        }
        McpAction::LspHover { .. }
        | McpAction::LspDefinition { .. }
        | McpAction::LspReferences { .. }
        | McpAction::LspRename { .. }
        | McpAction::LspCodeAction { .. }
        | McpAction::LspFormat { .. } => dispatch_mcp_lsp(action, sink),
        _ => dispatch_mcp_todo(action, desktop),
    }
}

fn dispatch_mcp_split(action: &McpAction, sink: &txv_core::prelude::EventSink) -> Result<serde_json::Value, String> {
    match action {
        McpAction::SplitVertical { file } => mcp_split(sink, true, file.clone()),
        McpAction::SplitHorizontal { file } => mcp_split(sink, false, file.clone()),
        McpAction::SplitClose => {
            sink.push_command(CM_SPLIT_CLOSE, None);
            Ok(serde_json::json!({"split": "closed"}))
        }
        McpAction::SplitFocus => {
            sink.push_command(CM_SPLIT_FOCUS, None);
            Ok(serde_json::json!({"split": "focus_switched"}))
        }
        McpAction::SplitOpen { path } => {
            let req = OpenFileRequest {
                path: PathBuf::from(path),
                line: None,
                col: None,
                diff: false,
            };
            sink.push_command(CM_OPEN_IN_SPLIT, Some(Box::new(req)));
            Ok(serde_json::json!({"split": "opened"}))
        }
        McpAction::SplitLinked { on } => {
            sink.push_command(CM_SPLIT_LINKED, Some(Box::new(*on)));
            Ok(serde_json::json!({"linked_scroll": on}))
        }
        _ => Err("Not a split action".to_string()),
    }
}

fn dispatch_mcp_git(action: &McpAction, sink: &txv_core::prelude::EventSink) -> Result<serde_json::Value, String> {
    match action {
        McpAction::GitStage { file } => {
            sink.push_command(CM_GIT_STAGE, Some(Box::new(file.clone())));
            Ok(serde_json::json!({"staged": file}))
        }
        McpAction::GitUnstage { file } => {
            sink.push_command(CM_GIT_UNSTAGE, Some(Box::new(file.clone())));
            Ok(serde_json::json!({"unstaged": file}))
        }
        McpAction::GitCommit { message } => {
            sink.push_command(CM_GIT_COMMIT, Some(Box::new(message.clone())));
            Ok(serde_json::json!({"committed": message}))
        }
        _ => Err("Not a git action".to_string()),
    }
}

fn dispatch_mcp_lsp(action: &McpAction, sink: &txv_core::prelude::EventSink) -> Result<serde_json::Value, String> {
    match action {
        McpAction::LspHover { .. } => {
            sink.push_command(CM_LSP_HOVER, None);
            Ok(serde_json::json!({"triggered": "hover"}))
        }
        McpAction::LspDefinition { .. } => {
            sink.push_command(CM_LSP_GOTO_DEF, None);
            Ok(serde_json::json!({"triggered": "definition"}))
        }
        McpAction::LspReferences { .. } => {
            sink.push_command(CM_LSP_FIND_REFS, None);
            Ok(serde_json::json!({"triggered": "references"}))
        }
        McpAction::LspRename { new_name, .. } => {
            sink.push_command(CM_LSP_RENAME, Some(Box::new(new_name.clone())));
            Ok(serde_json::json!({"triggered": "rename", "new_name": new_name}))
        }
        McpAction::LspCodeAction { .. } => {
            sink.push_command(CM_CODE_ACTION, None);
            Ok(serde_json::json!({"triggered": "code-action"}))
        }
        McpAction::LspFormat { .. } => {
            sink.push_command(CM_LSP_FORMAT, None);
            Ok(serde_json::json!({"triggered": "format"}))
        }
        _ => Err("Not an LSP action".to_string()),
    }
}

fn dispatch_mcp_todo(
    action: &McpAction,
    desktop: &mut txv_widgets::tiled_workspace::TiledWorkspace,
) -> Result<serde_json::Value, String> {
    let Some(panel) = desktop.panel_mut(SlotId::Left as usize) else {
        return Err("Left panel unavailable".to_string());
    };
    let todo_view = panel
        .view_at_mut(2)
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<TodoTreeView>());
    match todo_view {
        Some(tv) => tv.mcp_action(action),
        None => Err("Todo view not found".to_string()),
    }
}
