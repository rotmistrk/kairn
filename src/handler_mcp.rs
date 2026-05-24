//! MCP command drain — dispatches MCP write requests to app state.

use txv_core::program::CommandContext;

use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};
use crate::handler_mcp_helpers::*;

/// Drain MCP write commands and execute them on the live app state.
pub fn drain_mcp(ctx: &mut CommandContext, state: &mut AppState) {
    let Some(ref queue) = state.mcp_commands else {
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
        let result = match &req.action {
            crate::mcp::commands::McpAction::OpenFile { path } => mcp_open_file(desktop, state, ctx.sink, path),
            crate::mcp::commands::McpAction::CreateFile { path, content } => {
                mcp_create_file(desktop, state, ctx.sink, path, content)
            }
            crate::mcp::commands::McpAction::CloseTab { name } => mcp_close_tab(desktop, state, name),
            crate::mcp::commands::McpAction::EditBuffer {
                name,
                start_line,
                end_line,
                text,
            } => crate::handler_mcp_edit::mcp_edit_buffer(desktop, name, *start_line, *end_line, text),
            crate::mcp::commands::McpAction::InsertText { name, line, col, text } => {
                crate::handler_mcp_edit::mcp_insert_text(desktop, name, *line, *col, text)
            }
            crate::mcp::commands::McpAction::SetCursor { name, line, col } => {
                crate::handler_mcp_edit::mcp_set_cursor(desktop, name, *line, *col)
            }
            crate::mcp::commands::McpAction::SaveFile { name } => crate::handler_mcp_edit::mcp_save_file(desktop, name),
            crate::mcp::commands::McpAction::GetDiagnostics { name } => {
                crate::handler_mcp_edit::mcp_get_diagnostics(desktop, name)
            }
            crate::mcp::commands::McpAction::GetBuildErrors => crate::handler_mcp_build::mcp_get_build_errors(state),
            crate::mcp::commands::McpAction::SearchProject { pattern } => {
                crate::handler_mcp_build::mcp_search_project(state, pattern)
            }
            crate::mcp::commands::McpAction::RunBuild { command } => {
                crate::handler_mcp_build::mcp_run_build(state, ctx.sink, command)
            }
            crate::mcp::commands::McpAction::SplitVertical { file } => mcp_split(ctx.sink, true, file.clone()),
            crate::mcp::commands::McpAction::SplitHorizontal { file } => mcp_split(ctx.sink, false, file.clone()),
            crate::mcp::commands::McpAction::SplitClose => {
                ctx.sink.push_command(crate::commands::CM_SPLIT_CLOSE, None);
                Ok(serde_json::json!({"split": "closed"}))
            }
            crate::mcp::commands::McpAction::SplitFocus => {
                ctx.sink.push_command(crate::commands::CM_SPLIT_FOCUS, None);
                Ok(serde_json::json!({"split": "focus_switched"}))
            }
            crate::mcp::commands::McpAction::SplitOpen { path } => {
                let req = crate::commands::OpenFileRequest {
                    path: std::path::PathBuf::from(path),
                    line: None,
                    col: None,
                    diff: false,
                };
                ctx.sink
                    .push_command(crate::commands::CM_OPEN_IN_SPLIT, Some(Box::new(req)));
                Ok(serde_json::json!({"split": "opened"}))
            }
            crate::mcp::commands::McpAction::SplitLinked { on } => {
                ctx.sink
                    .push_command(crate::commands::CM_SPLIT_LINKED, Some(Box::new(*on)));
                Ok(serde_json::json!({"linked_scroll": on}))
            }
            crate::mcp::commands::McpAction::DiffRevert { name } => mcp_diff_revert(desktop, name),
            crate::mcp::commands::McpAction::LspControl { command } => {
                let msg = crate::handler_lsp_cmd::handle_lsp_command(command, state);
                Ok(serde_json::json!({"result": msg}))
            }
            crate::mcp::commands::McpAction::SendTerminalInput { name, input } => {
                mcp_send_terminal_input(desktop, name, input)
            }
            crate::mcp::commands::McpAction::GitStage { file } => {
                ctx.sink
                    .push_command(crate::commands::CM_GIT_STAGE, Some(Box::new(file.clone())));
                Ok(serde_json::json!({"staged": file}))
            }
            crate::mcp::commands::McpAction::GitUnstage { file } => {
                ctx.sink
                    .push_command(crate::commands::CM_GIT_UNSTAGE, Some(Box::new(file.clone())));
                Ok(serde_json::json!({"unstaged": file}))
            }
            crate::mcp::commands::McpAction::GitCommit { message } => {
                ctx.sink
                    .push_command(crate::commands::CM_GIT_COMMIT, Some(Box::new(message.clone())));
                Ok(serde_json::json!({"committed": message}))
            }
            crate::mcp::commands::McpAction::LspHover { .. } => {
                ctx.sink.push_command(crate::commands::CM_LSP_HOVER, None);
                Ok(serde_json::json!({"triggered": "hover"}))
            }
            crate::mcp::commands::McpAction::LspDefinition { .. } => {
                ctx.sink.push_command(crate::commands::CM_LSP_GOTO_DEF, None);
                Ok(serde_json::json!({"triggered": "definition"}))
            }
            crate::mcp::commands::McpAction::LspReferences { .. } => {
                ctx.sink.push_command(crate::commands::CM_LSP_FIND_REFS, None);
                Ok(serde_json::json!({"triggered": "references"}))
            }
            crate::mcp::commands::McpAction::LspRename { new_name, .. } => {
                ctx.sink
                    .push_command(crate::commands::CM_LSP_RENAME, Some(Box::new(new_name.clone())));
                Ok(serde_json::json!({"triggered": "rename", "new_name": new_name}))
            }
            crate::mcp::commands::McpAction::LspCodeAction { .. } => {
                ctx.sink.push_command(crate::commands::CM_CODE_ACTION, None);
                Ok(serde_json::json!({"triggered": "code-action"}))
            }
            crate::mcp::commands::McpAction::Undo { name } => mcp_undo_redo(desktop, name, true),
            crate::mcp::commands::McpAction::Redo { name } => mcp_undo_redo(desktop, name, false),
            crate::mcp::commands::McpAction::EvalTcl { script } => mcp_eval_tcl(state, script),
            _ => {
                let Some(panel) = desktop.panel_mut(SlotId::Left as usize) else {
                    return;
                };
                let todo_view = panel
                    .view_at_mut(2)
                    .and_then(|v| v.as_any_mut())
                    .and_then(|a| a.downcast_mut::<crate::views::todo_tree::TodoTreeView>());
                match todo_view {
                    Some(tv) => tv.mcp_action(&req.action),
                    None => Err("Todo view not found".to_string()),
                }
            }
        };
        let _ = req.reply.send(result);
    }
}
