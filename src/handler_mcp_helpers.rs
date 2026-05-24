//! MCP helper functions — file/tab/terminal operations.

use txv_core::prelude::*;

use crate::desktop::{close_tab_by_title, focus_tab_by_title, SlotId};
use crate::handler::AppState;
use txv_widgets::tiled_workspace::TiledWorkspace;

pub(crate) fn mcp_open_file(
    desktop: &mut TiledWorkspace,
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
            focus_tab_by_title(desktop, SlotId::Center, &title);
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

pub(crate) fn mcp_create_file(
    desktop: &mut TiledWorkspace,
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

pub(crate) fn mcp_close_tab(
    desktop: &mut TiledWorkspace,
    state: &mut AppState,
    name: &str,
) -> Result<serde_json::Value, String> {
    if close_tab_by_title(desktop, SlotId::Center, name) {
        state.broker.close(&state.root_dir.join(name).to_string_lossy());
        Ok(serde_json::json!({"closed": name}))
    } else {
        Err(format!("Tab not found: {name}"))
    }
}

pub(crate) fn mcp_split(sink: &EventSink, vertical: bool, file: Option<String>) -> Result<serde_json::Value, String> {
    let req = crate::commands::SplitRequest { vertical, file };
    sink.push_command(crate::commands::CM_SPLIT, Some(Box::new(req)));
    let dir = if vertical {
        "vertical"
    } else {
        "horizontal"
    };
    Ok(serde_json::json!({"split": dir}))
}

pub(crate) fn find_editor<'a>(
    desktop: &'a mut TiledWorkspace,
    name: &str,
) -> Result<&'a mut crate::views::editor::EditorView, String> {
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return Err("Panel not found".to_string());
    };
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

pub(crate) fn mcp_diff_revert(desktop: &mut TiledWorkspace, name: &str) -> Result<serde_json::Value, String> {
    let editor = find_editor(desktop, name)?;
    let msg = editor.revert_hunk()?;
    Ok(serde_json::json!({"result": msg}))
}

pub(crate) fn mcp_send_terminal_input(
    desktop: &mut TiledWorkspace,
    name: &str,
    input: &str,
) -> Result<serde_json::Value, String> {
    let Some(panel) = desktop.panel_mut(SlotId::Tools as usize) else {
        return Err("Right panel not found".to_string());
    };
    for i in 0..panel.tab_count() {
        if panel.tab_title(i) == Some(name) {
            let view = panel.view_at_mut(i).ok_or("View not accessible")?;
            let any = view.as_any_mut().ok_or("View has no Any")?;
            if let Some(term) = any.downcast_mut::<crate::views::terminal::TerminalView>() {
                term.write_input(input.as_bytes());
                return Ok(serde_json::json!({"sent": true}));
            }
            return Err(format!("Tab '{name}' is not a terminal"));
        }
    }
    Err(format!("Terminal tab not found: {name}"))
}

pub(crate) fn mcp_undo_redo(desktop: &mut TiledWorkspace, name: &str, undo: bool) -> Result<serde_json::Value, String> {
    let editor = if name.is_empty() {
        let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
            return Err("Panel not found".to_string());
        };
        panel
            .active_view_mut()
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_mut::<crate::views::editor::EditorView>())
            .ok_or("No active editor")?
    } else {
        find_editor(desktop, name)?
    };
    if undo {
        editor.undo();
        Ok(serde_json::json!({"action": "undo"}))
    } else {
        editor.redo();
        Ok(serde_json::json!({"action": "redo"}))
    }
}

pub(crate) fn mcp_eval_tcl(state: &mut AppState, script: &str) -> Result<serde_json::Value, String> {
    match state.script.eval(script) {
        Ok(result) => Ok(serde_json::json!({"result": result})),
        Err(e) => Err(format!("Tcl error: {e}")),
    }
}
