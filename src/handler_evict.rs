//! Handler logic for tab eviction — LRU close via editor's own prompt.

use txv_core::prelude::*;

use crate::eviction::PendingTab;
use crate::handler::AppState;
use crate::layout_group::{LayoutGroup, SlotId};
use crate::views::editor::EditorView;

/// Try to insert a tab, handling eviction if at capacity.
/// Returns true if the tab was inserted immediately.
/// Returns false if the LRU tab's close prompt was triggered (pending).
pub fn try_insert_tab(
    desktop: &mut LayoutGroup,
    state: &mut AppState,
    slot: SlotId,
    title: String,
    view: Box<dyn View>,
) -> bool {
    // Clear stale pending_tab if present (user cancelled previous eviction)
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
        desktop.panel_mut(slot).remove_tab(lru_idx);
        desktop.insert_tab(slot, &title, view);
        return true;
    }
    // Dirty LRU: activate it and trigger its close prompt
    desktop.panel_mut(slot).set_active(lru_idx);
    trigger_close_prompt(desktop, slot, lru_idx);
    state.pending_tab = Some(PendingTab { slot, title, view });
    false
}

/// Trigger the editor's own close prompt on the LRU tab.
fn trigger_close_prompt(desktop: &mut LayoutGroup, slot: SlotId, idx: usize) {
    let panel = desktop.panel_mut(slot);
    if let Some(view) = panel.view_at_mut(idx) {
        if let Some(any) = view.as_any_mut() {
            if let Some(editor) = any.downcast_mut::<EditorView>() {
                editor.request_close();
            }
        }
    }
}

/// Called from CM_FILE_CLOSED handler.
/// If a pending tab exists, the editor confirmed close — we remove
/// the LRU tab and insert the pending tab.
pub fn complete_pending_insert(desktop: &mut LayoutGroup, state: &mut AppState) {
    let Some(pending) = state.pending_tab.take() else {
        return;
    };
    let slot = pending.slot;
    // Remove the active tab (the LRU tab whose close was just confirmed).
    // Use remove_tab (force) since the editor already handled save/discard.
    let active = desktop.panel(slot).active_index();
    desktop.panel_mut(slot).remove_tab(active);
    // Insert the pending tab
    desktop.insert_tab(slot, &pending.title, pending.view);
}
