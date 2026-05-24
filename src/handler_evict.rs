//! Handler logic for tab eviction — LRU close via ConfirmItem prompt.

use txv_core::prelude::*;

use crate::app_state::AppState;
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};
use crate::desktop::SlotId;
use crate::eviction::PendingTab;
use crate::views::editor::EditorView;
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
    state.pending_tab = None;

    let max = state.settings.max_tabs as usize;
    let Some(panel) = desktop.panel(slot as usize) else {
        return false;
    };
    let count = panel.tab_count();
    if max == 0 || count < max {
        desktop.insert_tab(slot as usize, &title, view);
        return true;
    }
    let Some(panel) = desktop.panel(slot as usize) else {
        return false;
    };
    let lru_idx = match panel.lru_index() {
        Some(i) => i,
        None => {
            desktop.insert_tab(slot as usize, &title, view);
            return true;
        }
    };
    let Some(panel) = desktop.panel(slot as usize) else {
        return false;
    };
    if panel.can_close_tab(lru_idx) == CloseResult::Ok {
        if state.settings.editor_defaults.autosave {
            if let Some(panel) = desktop.panel_mut(slot as usize) {
                if let Some(view) = panel.view_at_mut(lru_idx) {
                    if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                        if editor.editor.buf().is_dirty() {
                            let _ = editor.save();
                        }
                    }
                }
            }
        }
        if let Some(panel) = desktop.panel(slot as usize) {
            if let Some(tab_title) = panel.tab_title(lru_idx).map(String::from) {
                state.broker.close(&tab_title);
            }
        }
        if let Some(panel) = desktop.panel_mut(slot as usize) {
            panel.remove_tab(lru_idx);
        }
        desktop.insert_tab(slot as usize, &title, view);
        return true;
    }
    if let Some(panel) = desktop.panel_mut(slot as usize) {
        panel.set_active(lru_idx);
    }
    trigger_close_prompt(desktop, sink, slot, lru_idx);
    state.pending_tab = Some(PendingTab { slot, title, view });
    false
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
    let Some(pending) = state.pending_tab.take() else {
        return;
    };
    let slot = pending.slot;
    if let Some(panel) = desktop.panel_mut(slot as usize) {
        let active = panel.active_index();
        panel.remove_tab(active);
    }
    desktop.insert_tab(slot as usize, &pending.title, pending.view);
}
