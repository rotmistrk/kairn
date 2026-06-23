//! Handler logic for tab eviction — LRU close via ConfirmItem prompt.

use txv_core::prelude::*;

use crate::app_state::AppState;
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};
use crate::desktop::SlotId;
use crate::eviction::PendingTab;
use crate::views::editor::{EditorView, EditorViewExt};
use txv_widgets::tiled_workspace::TiledWorkspace;

/// Try to insert a tab, handling eviction if at capacity.
/// Returns true if the tab was inserted immediately.
/// Returns false if the LRU tab's close prompt was triggered (pending).
pub fn try_insert_tab(
    desktop: &mut TiledWorkspace,
    state: &mut AppState,
    sink: &EventSink,
    slot: SlotId,
    title: String,
    view: Box<dyn View>,
) -> bool {
    state.pending_mut().set_pending_tab(None);

    let max = state.settings().max_tabs() as usize;
    let Some(panel) = desktop.panel(slot as usize) else {
        return false;
    };
    let count = panel.tab_count();
    if max == 0 || count < max {
        desktop.insert_tab(slot as usize, &title, view);
        return true;
    }

    let lru_idx = match desktop.panel(slot as usize).and_then(|p| p.lru_index()) {
        Some(i) => i,
        None => {
            desktop.insert_tab(slot as usize, &title, view);
            return true;
        }
    };

    if can_evict_lru(desktop, state, slot, lru_idx) {
        evict_and_insert(desktop, state, slot, lru_idx, title, view);
        return true;
    }

    if let Some(panel) = desktop.panel_mut(slot as usize) {
        panel.set_active(lru_idx);
    }
    trigger_close_prompt(desktop, sink, slot, lru_idx);
    state
        .pending_mut()
        .set_pending_tab(Some(PendingTab { slot, title, view }));
    false
}

fn can_evict_lru(desktop: &mut TiledWorkspace, _state: &AppState, slot: SlotId, lru_idx: usize) -> bool {
    desktop
        .panel(slot as usize)
        .is_some_and(|p| p.can_close_tab(lru_idx) == CloseResult::Ok)
}

fn evict_and_insert(
    desktop: &mut TiledWorkspace,
    state: &mut AppState,
    slot: SlotId,
    lru_idx: usize,
    title: String,
    view: Box<dyn View>,
) {
    if state.settings().editor_defaults().autosave() {
        autosave_tab(desktop, slot, lru_idx);
    }
    if let Some(panel) = desktop.panel_mut(slot as usize) {
        let abs_path = panel
            .view_at_mut(lru_idx)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_ref::<EditorView>())
            .map(|ev| ev.path().to_string_lossy().to_string());
        if let Some(p) = abs_path {
            state.workspace_mut().broker_mut().close(&p);
        }
        panel.remove_tab(lru_idx);
    }
    desktop.insert_tab(slot as usize, &title, view);
}

fn autosave_tab(desktop: &mut TiledWorkspace, slot: SlotId, idx: usize) {
    let editor = desktop
        .panel_mut(slot as usize)
        .and_then(|p| p.view_at_mut(idx))
        .and_then(|v| v.as_any_mut())
        .and_then(|a| a.downcast_mut::<EditorView>());
    if let Some(editor) = editor {
        if editor.editor().buf().is_dirty() {
            let _ = editor.save();
        }
    }
}

fn trigger_close_prompt(desktop: &mut TiledWorkspace, sink: &EventSink, slot: SlotId, idx: usize) {
    let Some(panel) = desktop.panel_mut(slot as usize) else {
        return;
    };
    if let Some(view) = panel.view_at_mut(idx) {
        if let Some(any) = view.as_any_mut() {
            if let Some(editor) = any.downcast_mut::<EditorView>() {
                editor.request_close();
                let path = editor.path().to_string_lossy().to_string();
                sink.push_command(
                    CM_SET_CONFIRM_CONTEXT,
                    Some(Box::new(ConfirmContext::EditorClose(path))),
                );
                sink.push_command(
                    CM_CONFIRM,
                    Some(Box::new("Save changes? [y]es [n]o [Esc]cancel".to_string())),
                );
            }
        }
    }
}

/// Called from CM_FILE_CLOSED handler.
pub fn complete_pending_insert(desktop: &mut TiledWorkspace, state: &mut AppState) {
    let Some(pending) = state.pending_mut().take_pending_tab() else {
        return;
    };
    let slot = pending.slot;
    if let Some(panel) = desktop.panel_mut(slot as usize) {
        let active = panel.active_index();
        panel.remove_tab(active);
    }
    desktop.insert_tab(slot as usize, &pending.title, pending.view);
}
