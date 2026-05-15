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
            crate::mcp::commands::McpAction::EditBuffer {
                name,
                start_line,
                end_line,
                text,
            } => mcp_edit_buffer(desktop, name, *start_line, *end_line, text),
            crate::mcp::commands::McpAction::InsertText { name, line, col, text } => {
                mcp_insert_text(desktop, name, *line, *col, text)
            }
            crate::mcp::commands::McpAction::SetCursor { name, line, col } => {
                mcp_set_cursor(desktop, name, *line, *col)
            }
            crate::mcp::commands::McpAction::SaveFile { name } => mcp_save_file(desktop, name),
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

/// Find an editor view by tab name in the center panel.
fn find_editor<'a>(
    desktop: &'a mut crate::layout_group::LayoutGroup,
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

fn mcp_edit_buffer(
    desktop: &mut crate::layout_group::LayoutGroup,
    name: &str,
    start_line: usize,
    end_line: usize,
    text: &str,
) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    let buf = &mut editor.editor.buffer;
    let line_count = buf.line_count();
    let start = start_line.min(line_count);
    let end = end_line.min(line_count);
    if start > end {
        return Err("start_line > end_line".to_string());
    }
    // Delete the range, then insert new text
    let start_offset = buf.line_col_to_offset(start, 0).unwrap_or(0);
    let end_offset = if end >= line_count {
        buf.content().len()
    } else {
        buf.line_col_to_offset(end, 0).unwrap_or(buf.content().len())
    };
    if end_offset > start_offset {
        buf.delete(start_offset, end_offset);
    }
    if !text.is_empty() {
        let insert_at = buf.line_col_to_offset(start, 0).unwrap_or(0);
        buf.insert(insert_at, text);
    }
    Ok(serde_json::json!({"edited": name}))
}

fn mcp_insert_text(
    desktop: &mut crate::layout_group::LayoutGroup,
    name: &str,
    line: usize,
    col: usize,
    text: &str,
) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    editor.editor.buffer.insert_at(line, col, text);
    Ok(serde_json::json!({"inserted": text.len()}))
}

fn mcp_set_cursor(
    desktop: &mut crate::layout_group::LayoutGroup,
    name: &str,
    line: usize,
    col: usize,
) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    editor.goto(line as u32, col as u32);
    Ok(serde_json::json!({"cursor": {"line": line, "col": col}}))
}

fn mcp_save_file(desktop: &mut crate::layout_group::LayoutGroup, name: &str) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    editor.save().map_err(|e| format!("Save failed: {e}"))?;
    Ok(serde_json::json!({"saved": name}))
}
