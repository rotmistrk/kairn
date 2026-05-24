//! Slot identifiers and workspace convenience functions.

use txv_core::prelude::View;
use txv_widgets::tiled_workspace::TiledWorkspace;

pub use txv_widgets::tiled_workspace::types::LayoutMode;

/// Identifies one of the three panel slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(usize)]
pub enum SlotId {
    Left = 0,
    Center = 1,
    Tools = 2,
}

pub const PANEL_COUNT: usize = 3;

pub fn slot_from(idx: usize) -> SlotId {
    match idx {
        0 => SlotId::Left,
        1 => SlotId::Center,
        _ => SlotId::Tools,
    }
}

pub fn find_view_mut<T: View + 'static>(ws: &mut TiledWorkspace, slot: SlotId) -> Option<&mut T> {
    let panel = ws.panel_mut(slot as usize)?;
    let count = panel.tab_count();
    let idx = (0..count).find(|&i| {
        panel
            .view_at_mut(i)
            .and_then(|v| v.as_any_mut())
            .is_some_and(|a| a.downcast_ref::<T>().is_some())
    })?;
    let panel = ws.panel_mut(slot as usize)?;
    let view = panel.view_at_mut(idx)?;
    view.as_any_mut()?.downcast_mut::<T>()
}

pub fn focus_view_mut<T: View + 'static>(ws: &mut TiledWorkspace, slot: SlotId) -> Option<&mut T> {
    let panel = ws.panel_mut(slot as usize)?;
    let count = panel.tab_count();
    let idx = (0..count).find(|&i| {
        panel
            .view_at_mut(i)
            .and_then(|v| v.as_any_mut())
            .is_some_and(|a| a.downcast_ref::<T>().is_some())
    })?;
    let panel = ws.panel_mut(slot as usize)?;
    panel.set_active(idx);
    let view = panel.active_view_mut()?;
    view.as_any_mut()?.downcast_mut::<T>()
}

pub fn insert_tab(ws: &mut TiledWorkspace, slot: SlotId, title: impl Into<String>, view: Box<dyn View>) {
    let id = slot as usize;
    ws.insert_tab(id, title, view);
    if ws.is_hidden(id) {
        ws.set_hidden(id, false);
    }
}

#[allow(deprecated)]
pub fn focus_tab_by_title(ws: &mut TiledWorkspace, slot: SlotId, title: &str) -> bool {
    ws.panel_mut(slot as usize).is_some_and(|p| p.focus_tab_by_title(title))
}

#[allow(deprecated)]
pub fn close_tab_by_title(ws: &mut TiledWorkspace, slot: SlotId, title: &str) -> bool {
    ws.panel_mut(slot as usize).is_some_and(|p| p.close_tab_by_title(title))
}

pub fn active_tab_title(ws: &TiledWorkspace, slot: SlotId) -> Option<&str> {
    ws.panel(slot as usize)?.active_title()
}

pub fn next_tab_name(ws: &TiledWorkspace, slot: SlotId, prefix: &str) -> String {
    ws.panel(slot as usize)
        .map_or_else(|| format!("{prefix} 1"), |p| p.next_tab_name(prefix))
}
