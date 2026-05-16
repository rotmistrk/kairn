//! Handler logic for tab eviction — LRU close via ConfirmItem prompt.

use txv_core::prelude::*;

use crate::app_state::AppState;
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};
use crate::eviction::PendingTab;
use crate::layout_group::{LayoutGroup, SlotId};
use crate::views::editor::EditorView;

/// Try to insert a tab, handling eviction if at capacity.
/// Returns true if the tab was inserted immediately.
/// Returns false if the LRU tab's close prompt was triggered (pending).
pub fn try_insert_tab(
    desktop: &mut LayoutGroup,
    state: &mut AppState,
    sink: &EventSink,
    slot: SlotId,
    title: String,
    view: Box<dyn View>,
) -> bool {
    state.pending_tab = None;

    let max = state.settings.max_tabs as usize;
    let count = desktop.panel(slot).tab_count();
    if max == 0 || count < max {
        desktop.insert_tab(slot, &title, view);
        return true;
    }
    let lru_idx = match desktop.panel(slot).lru_index() {
        Some(i) => i,
        None => {
            desktop.insert_tab(slot, &title, view);
            return true;
        }
    };
    if desktop.panel(slot).can_close_tab(lru_idx) == CloseResult::Ok {
        // Save if autosave is on and buffer is dirty
        if state.settings.editor_defaults.autosave {
            if let Some(view) = desktop.panel_mut(slot).view_at_mut(lru_idx) {
                if let Some(editor) = view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                    if editor.editor.buf().is_dirty() {
                        let _ = editor.save();
                    }
                }
            }
        }
        // Unregister from broker before removing
        if let Some(tab_title) = desktop.panel(slot).tab_title(lru_idx).map(String::from) {
            let full_path = state.root_dir.join(&tab_title);
            state.broker.close(&full_path.to_string_lossy());
            state.broker.close(&tab_title);
        }
        desktop.panel_mut(slot).remove_tab(lru_idx);
        desktop.insert_tab(slot, &title, view);
        return true;
    }
    desktop.panel_mut(slot).set_active(lru_idx);
    trigger_close_prompt(desktop, sink, slot, lru_idx);
    state.pending_tab = Some(PendingTab { slot, title, view });
    false
}

fn trigger_close_prompt(desktop: &mut LayoutGroup, sink: &EventSink, slot: SlotId, idx: usize) {
    let panel = desktop.panel_mut(slot);
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
pub fn complete_pending_insert(desktop: &mut LayoutGroup, state: &mut AppState) {
    let Some(pending) = state.pending_tab.take() else {
        return;
    };
    let slot = pending.slot;
    let active = desktop.panel(slot).active_index();
    desktop.panel_mut(slot).remove_tab(active);
    desktop.insert_tab(slot, &pending.title, pending.view);
}
