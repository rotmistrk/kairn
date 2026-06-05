//! MCP helper functions — file/tab/terminal operations.

use std::fs;
use std::path::{Path, PathBuf};

use txv_core::prelude::*;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::broker::OpenResult;
use crate::commands::{RootsChangedData, SplitRequest, CM_ROOTS_CHANGED, CM_SPLIT};
use crate::desktop::{close_tab_by_title, focus_editor_by_path, SlotId};
use crate::handler::AppState;
use crate::handler_evict::try_insert_tab;
use crate::views::editor::EditorView;
use crate::views::terminal::TerminalView;

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
        OpenResult::AlreadyOpen { .. } => {
            focus_editor_by_path(desktop, &path_str);
        }
        OpenResult::Opened => {
            let defaults = &state.settings.editor_defaults;
            let theme = state.current_syntax_theme();
            let view: Box<dyn View> = match EditorView::open_with_theme(&path, defaults, theme) {
                Ok(mut ed) => {
                    ed.set_root_dir(state.roots().root_for(&path).path().to_path_buf());
                    ed.editor_mut().set_shared_register(state.shared_register.clone());
                    Box::new(ed)
                }
                Err(_) => Box::new(EditorView::new_file(&path, defaults)),
            };
            try_insert_tab(desktop, state, sink, SlotId::Center, title, view);
        }
    }
    Ok(serde_json::json!({"opened": rel_path}))
}

pub(crate) fn mcp_highlight_code(
    desktop: &mut TiledWorkspace,
    state: &mut AppState,
    sink: &EventSink,
    rel_path: &str,
    ranges: &[(u32, u32)],
) -> Result<serde_json::Value, String> {
    use crate::editor::ephemeral::HighlightOwner;
    use crate::editor::ephemeral_range::EphemeralRange;
    mcp_open_file(desktop, state, sink, rel_path)?;
    let panel = desktop.panel_mut(SlotId::Center as usize).ok_or("No center panel")?;
    let view = panel.active_view_mut().ok_or("No active view")?;
    if let Some(ev) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
        let eph: Vec<EphemeralRange> = ranges
            .iter()
            .map(|&(s, e)| EphemeralRange::line_range(s.saturating_sub(1) as usize, e.saturating_sub(1) as usize))
            .collect();
        ev.editor_mut().ephemeral.set(eph, HighlightOwner::Transient);
        if let Some(&(start, _)) = ranges.first() {
            ev.goto(start.saturating_sub(1), 0);
        }
    }
    Ok(serde_json::json!({"highlighted": rel_path, "ranges": ranges.len()}))
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
        fs::create_dir_all(parent).map_err(|e| format!("Cannot create dirs: {e}"))?;
    }
    fs::write(&path, content).map_err(|e| format!("Cannot write file: {e}"))?;
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
    let req = SplitRequest { vertical, file };
    sink.push_command(CM_SPLIT, Some(Box::new(req)));
    let dir = if vertical {
        "vertical"
    } else {
        "horizontal"
    };
    Ok(serde_json::json!({"split": dir}))
}

pub(crate) fn find_editor<'a>(desktop: &'a mut TiledWorkspace, name: &str) -> Result<&'a mut EditorView, String> {
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return Err("Panel not found".to_string());
    };
    // Match by title first, then by path suffix
    let idx = (0..panel.tab_count())
        .find(|&i| panel.tab_title(i) == Some(name))
        .or_else(|| {
            (0..panel.tab_count()).find(|&i| {
                panel
                    .view_at_mut(i)
                    .and_then(|v| v.as_any_mut())
                    .and_then(|a| a.downcast_ref::<EditorView>())
                    .is_some_and(|ev| {
                        let p = ev.path().to_string_lossy();
                        p.ends_with(name) || p == name
                    })
            })
        })
        .ok_or_else(|| format!("Tab not found: {name}"))?;
    let view = panel.view_at_mut(idx).ok_or("View not accessible")?;
    let any = view.as_any_mut().ok_or("View has no Any")?;
    any.downcast_mut::<EditorView>()
        .ok_or_else(|| format!("Tab '{name}' is not an editor"))
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
            if let Some(term) = any.downcast_mut::<TerminalView>() {
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
            .and_then(|a| a.downcast_mut::<EditorView>())
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

pub(crate) fn mcp_list_roots(state: &AppState) -> Result<serde_json::Value, String> {
    let roots: Vec<&str> = state.roots().all().iter().filter_map(|r| r.path.to_str()).collect();
    Ok(serde_json::json!({"roots": roots}))
}

pub(crate) fn mcp_add_root(state: &mut AppState, sink: &EventSink, path: &str) -> Result<serde_json::Value, String> {
    let p = PathBuf::from(path);
    if !p.is_dir() {
        return Err(format!("Not a directory: {path}"));
    }
    let added = state.roots_mut().add(p);
    if added {
        emit_roots_changed_sink(state, sink);
    }
    Ok(serde_json::json!({"added": added}))
}

pub(crate) fn mcp_remove_root(state: &mut AppState, sink: &EventSink, path: &str) -> Result<serde_json::Value, String> {
    let p = Path::new(path);
    let removed = state.roots_mut().remove(p);
    if removed {
        emit_roots_changed_sink(state, sink);
    }
    Ok(serde_json::json!({"removed": removed}))
}

fn emit_roots_changed_sink(state: &AppState, sink: &EventSink) {
    let data = RootsChangedData::from_roots(state.roots());
    sink.push_broadcast(CM_ROOTS_CHANGED, Some(Box::new(data)));
}
