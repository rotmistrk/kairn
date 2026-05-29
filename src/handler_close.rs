//! Close and quit handlers — checks unsaved buffers before closing/quitting.

use std::any::Any;

use txv_core::prelude::*;
use txv_core::program::CommandContext;
use txv_widgets::tiled_workspace::TiledWorkspace;

use crate::app_state::AppState;
use crate::commands::*;
use crate::handler::downcast_desktop;
use crate::slots::SlotId;
use crate::views::editor::EditorView;

/// Handle CM_APP_QUIT: always confirm before quitting.
pub(crate) fn handle_app_quit(ctx: &mut CommandContext, state: &mut AppState) {
    if let Some(desktop) = downcast_desktop(ctx.desktop) {
        let msg = if has_unsaved_buffers(desktop) {
            "Unsaved changes — quit? [y]es [n]o"
        } else {
            "Quit? [y]es [n]o"
        };
        state.confirm_context = Some(ConfirmContext::Quit);
        ctx.sink.push_command(CM_CONFIRM, Some(Box::new(msg.to_string())));
    }
}

/// Handle CM_TW_TAB_CLOSE / CM_TAB_CLOSE: check can_close, prompt if dirty.
pub(crate) fn handle_tab_close(ctx: &mut CommandContext, state: &mut AppState) {
    let (can_close, focused) = {
        let Some(desktop) = downcast_desktop(ctx.desktop) else {
            return;
        };
        let f = desktop.focused_panel();
        let cc = desktop
            .panel(f)
            .map(|p| p.can_close_tab(p.active_index()))
            .unwrap_or(CloseResult::Ok);
        (cc, f)
    };
    if can_close != CloseResult::Ok {
        if let Some(path) = active_editor_path(ctx, focused) {
            state.confirm_context = Some(ConfirmContext::EditorClose(path));
            ctx.sink.push_command(
                CM_CONFIRM,
                Some(Box::new("Save changes? [y]es [n]o [Esc]cancel".to_string())),
            );
        }
        return;
    }
    // Save autosave-pending buffer before closing (race: tick may not have fired)
    save_active_if_dirty(ctx, focused);
    close_active_tab(ctx, focused);
}

fn active_editor_path(ctx: &mut CommandContext, panel_id: usize) -> Option<String> {
    let desktop = downcast_desktop(ctx.desktop)?;
    let panel = desktop.panel_mut(panel_id)?;
    let view = panel.active_view_mut()?;
    let any = view.as_any_mut()?;
    let editor = any.downcast_ref::<EditorView>()?;
    Some(editor.path().to_string_lossy().to_string())
}

fn save_active_if_dirty(ctx: &mut CommandContext, panel_id: usize) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let Some(panel) = desktop.panel_mut(panel_id) else {
        return;
    };
    let Some(view) = panel.active_view_mut() else {
        return;
    };
    let Some(any) = view.as_any_mut() else {
        return;
    };
    if let Some(editor) = any.downcast_mut::<EditorView>() {
        if editor.editor.buf().is_dirty() {
            editor.save_now();
        }
    }
}

fn close_active_tab(ctx: &mut CommandContext, panel_id: usize) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    let title = desktop.panel(panel_id).and_then(|p| p.active_title().map(String::from));
    if let Some(panel) = desktop.panel_mut(panel_id) {
        panel.close_active();
    }
    ctx.sink
        .push_command(CM_FILE_CLOSED, title.map(|t| Box::new(t) as Box<dyn Any + Send>));
}

/// Check if any editor tab in the center panel has unsaved changes.
pub fn has_unsaved_buffers(desktop: &TiledWorkspace) -> bool {
    let Some(panel) = desktop.panel(SlotId::Center as usize) else {
        return false;
    };
    for i in 0..panel.tab_count() {
        if panel.can_close_tab(i) != CloseResult::Ok {
            return true;
        }
    }
    false
}

/// Handle CM_SAVE_ALL: save all dirty editor tabs in the center panel.
pub(crate) fn handle_save_all(ctx: &mut CommandContext) {
    let Some(desktop) = downcast_desktop(ctx.desktop) else {
        return;
    };
    save_all_dirty(desktop);
}

/// Save all dirty editor buffers in the center panel.
pub(crate) fn save_all_dirty(desktop: &mut TiledWorkspace) {
    let Some(panel) = desktop.panel_mut(SlotId::Center as usize) else {
        return;
    };
    for i in 0..panel.tab_count() {
        let Some(view) = panel.view_at_mut(i) else {
            continue;
        };
        let Some(any) = view.as_any_mut() else {
            continue;
        };
        let Some(editor) = any.downcast_mut::<EditorView>() else {
            continue;
        };
        if editor.editor.buf().is_dirty() {
            editor.save_now();
        }
    }
}
