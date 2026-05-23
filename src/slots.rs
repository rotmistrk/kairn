//! Slot identifiers and workspace convenience functions.

use txv_core::prelude::View;
use txv_widgets::tab_panel::TabPanel;
use txv_widgets::tiled_workspace::TiledWorkspace;

pub use txv_widgets::tiled_workspace::types::LayoutMode;

/// Identifies one of the four panel slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[repr(usize)]
pub enum SlotId {
    Left = 0,
    Center = 1,
    Right = 2,
    Bottom = 3,
}

pub const PANEL_COUNT: usize = 4;

pub fn slot_from(idx: usize) -> SlotId {
    match idx {
        0 => SlotId::Left,
        1 => SlotId::Center,
        2 => SlotId::Right,
        _ => SlotId::Bottom,
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

/// Convenience wrapper around TiledWorkspace for SlotId-based access.
/// All fields are public — this is a thin adapter, not encapsulation.
#[repr(transparent)]
pub struct Desktop {
    pub ws: TiledWorkspace,
}

impl std::ops::Deref for Desktop {
    type Target = TiledWorkspace;
    fn deref(&self) -> &TiledWorkspace {
        &self.ws
    }
}

impl std::ops::DerefMut for Desktop {
    fn deref_mut(&mut self) -> &mut TiledWorkspace {
        &mut self.ws
    }
}

impl View for Desktop {
    txv_core::delegate_view!(ws, override {});

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}

impl Desktop {
    pub fn new(ws: TiledWorkspace) -> Self {
        Self { ws }
    }

    /// Create a default desktop for testing.
    pub fn default_desktop() -> Self {
        Self::new(crate::build_desktop::create_workspace_shell())
    }

    pub fn panel(&self, slot: SlotId) -> &TabPanel {
        self.ws.panel(slot as usize).unwrap()
    }

    pub fn panel_mut(&mut self, slot: SlotId) -> &mut TabPanel {
        self.ws.panel_mut(slot as usize).unwrap()
    }

    pub fn focused_slot(&self) -> SlotId {
        slot_from(self.ws.focused_panel())
    }

    pub fn focus_slot(&mut self, slot: SlotId) {
        self.ws.focus_panel(slot as usize);
    }

    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, view: Box<dyn View>) {
        insert_tab(&mut self.ws, slot, title, view);
    }

    #[allow(deprecated)]
    pub fn focus_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.ws.panel_mut(slot as usize).unwrap().focus_tab_by_title(title)
    }

    #[allow(deprecated)]
    pub fn close_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.ws.panel_mut(slot as usize).unwrap().close_tab_by_title(title)
    }

    pub fn active_tab_title(&self, slot: SlotId) -> Option<&str> {
        self.ws.panel(slot as usize).unwrap().active_title()
    }

    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut dyn View> {
        self.ws.panel_mut(slot as usize).unwrap().active_view_mut()
    }

    pub fn tab_count(&self, slot: SlotId) -> usize {
        self.ws.panel(slot as usize).unwrap().tab_count()
    }

    pub fn set_active_tab(&mut self, slot: SlotId, index: usize) {
        self.ws.panel_mut(slot as usize).unwrap().set_active(index);
    }

    pub fn next_tab_name(&self, slot: SlotId, prefix: &str) -> String {
        self.ws.panel(slot as usize).unwrap().next_tab_name(prefix)
    }

    pub fn is_tall(&self) -> bool {
        !self.ws.is_wide()
    }

    pub fn find_view_mut<T: View + 'static>(&mut self, slot: SlotId) -> Option<&mut T> {
        find_view_mut(&mut self.ws, slot)
    }

    pub fn focus_view_mut<T: View + 'static>(&mut self, slot: SlotId) -> Option<&mut T> {
        focus_view_mut(&mut self.ws, slot)
    }

    pub fn rename_focused_tab(&mut self, new_user_part: &str) {
        let idx = self.ws.focused_panel();
        self.ws.panel_mut(idx).unwrap().rename_user_part(new_user_part);
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.ws.set_layout_mode(mode);
    }

    pub fn layout_mode(&self) -> LayoutMode {
        self.ws.layout_mode()
    }

    pub fn set_wide_threshold(&mut self, threshold: u16) {
        self.ws.set_wide_threshold(threshold);
    }

    pub fn layout_rects(&self) -> [txv_core::prelude::Rect; PANEL_COUNT] {
        let mut rects = [txv_core::prelude::Rect::default(); PANEL_COUNT];
        for (i, rect) in rects.iter_mut().enumerate() {
            if let Some(child) = self.ws.child(i) {
                *rect = child.bounds();
            }
        }
        rects
    }
}
