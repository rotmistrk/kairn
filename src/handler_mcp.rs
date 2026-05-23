//! MCP command drain — dispatches MCP write requests to app state.

use txv_core::prelude::*;
use txv_core::program::CommandContext;

use crate::desktop::SlotId;
use crate::handler::{downcast_desktop, AppState};

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
            _ => {
                let panel = desktop.panel_mut(SlotId::Left);
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

fn mcp_open_file(
    desktop: &mut crate::desktop::Desktop,
    state: &mut AppState,
    sink: &EventSink,
    rel_path: &str,
) -> Result<serde_json::Value, String> {
    let path = state.root_dir.join(rel_path);
    if !path.is_file() {
        return Err(format!("File not found: {rel_path}"));
    }
    let path_str = path.to_string_lossy().to_string();
    let title = rel_path.to_string();
    match state.broker.open(&path_str, SlotId::Center, 0) {
        crate::broker::OpenResult::AlreadyOpen { .. } => {
            desktop.focus_tab_by_title(SlotId::Center, &title);
        }
        crate::broker::OpenResult::Opened => {
            let defaults = &state.settings.editor_defaults;
            let theme = state.current_syntax_theme();
            let view: Box<dyn txv_core::view::View> =
                match crate::views::editor::EditorView::open_with_theme(&path, defaults, theme) {
                    Ok(mut ed) => {
                        ed.set_root_dir(state.root_dir.clone());
                        Box::new(ed)
                    }
                    Err(_) => Box::new(crate::views::editor::EditorView::new_file(&path, defaults)),
                };
            crate::handler_evict::try_insert_tab(desktop, state, sink, SlotId::Center, title, view);
        }
    }
    Ok(serde_json::json!({"opened": rel_path}))
}

fn mcp_create_file(
    desktop: &mut crate::desktop::Desktop,
    state: &mut AppState,
    sink: &EventSink,
    rel_path: &str,
    content: &str,
) -> Result<serde_json::Value, String> {
    let path = state.root_dir.join(rel_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create dirs: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("Cannot write file: {e}"))?;
    mcp_open_file(desktop, state, sink, rel_path)?;
    Ok(serde_json::json!({"created": rel_path}))
}

fn mcp_close_tab(
    desktop: &mut crate::desktop::Desktop,
    state: &mut AppState,
    name: &str,
) -> Result<serde_json::Value, String> {
    if desktop.close_tab_by_title(SlotId::Center, name) {
        state.broker.close(&state.root_dir.join(name).to_string_lossy());
        Ok(serde_json::json!({"closed": name}))
    } else {
        Err(format!("Tab not found: {name}"))
    }
}

fn mcp_split(sink: &EventSink, vertical: bool, file: Option<String>) -> Result<serde_json::Value, String> {
    let req = crate::commands::SplitRequest { vertical, file };
    sink.push_command(crate::commands::CM_SPLIT, Some(Box::new(req)));
    let dir = if vertical {
        "vertical"
    } else {
        "horizontal"
    };
    Ok(serde_json::json!({"split": dir}))
}

fn find_editor<'a>(
    desktop: &'a mut crate::desktop::Desktop,
    name: &str,
) -> Result<&'a mut crate::views::editor::EditorView, String> {
    let panel = desktop.panel_mut(SlotId::Center);
    for i in 0..panel.tab_count() {
        if panel.tab_title(i) == Some(name) {
            let view = panel.view_at_mut(i).ok_or("View not accessible")?;
            let any = view.as_any_mut().ok_or("View has no Any")?;
            return any
                .downcast_mut::<crate::views::editor::EditorView>()
                .ok_or_else(|| format!("Tab '{name}' is not an editor"));
        }
    }
    Err(format!("Tab not found: {name}"))
}

fn mcp_diff_revert(desktop: &mut crate::desktop::Desktop, name: &str) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    let msg = editor.revert_hunk()?;
    Ok(serde_json::json!({"result": msg}))
}
