//! MCP command drain — dispatches MCP write requests to app state.

use txv_core::program::CommandContext;

use crate::handler::{downcast_desktop, AppState};
use crate::layout_group::SlotId;

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
            crate::mcp::commands::McpAction::OpenFile { path } => mcp_open_file(desktop, state, ctx.queue, path),
            crate::mcp::commands::McpAction::CreateFile { path, content } => {
                mcp_create_file(desktop, state, ctx.queue, path, content)
            }
            crate::mcp::commands::McpAction::CloseTab { name } => mcp_close_tab(desktop, state, name),
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
    desktop: &mut crate::layout_group::LayoutGroup,
    state: &mut AppState,
    queue: &mut txv_core::view::EventQueue,
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
            crate::handler_evict::try_insert_tab(desktop, state, queue, SlotId::Center, title, view);
        }
    }
    Ok(serde_json::json!({"opened": rel_path}))
}

fn mcp_create_file(
    desktop: &mut crate::layout_group::LayoutGroup,
    state: &mut AppState,
    queue: &mut txv_core::view::EventQueue,
    rel_path: &str,
    content: &str,
) -> Result<serde_json::Value, String> {
    let path = state.root_dir.join(rel_path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("Cannot create dirs: {e}"))?;
    }
    std::fs::write(&path, content).map_err(|e| format!("Cannot write file: {e}"))?;
    mcp_open_file(desktop, state, queue, rel_path)?;
    Ok(serde_json::json!({"created": rel_path}))
}

fn mcp_close_tab(
    desktop: &mut crate::layout_group::LayoutGroup,
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
