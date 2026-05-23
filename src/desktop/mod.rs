//! Desktop — minimal View adapter for TiledWorkspace.
//!
//! Provides chrome drawing, command dispatch, and tick routing.
//! All panel access goes through workspace() directly.

use txv_core::prelude::*;
use txv_widgets::tab_panel::TabPanel;
use txv_widgets::tiled_workspace::types::{PanelConfig, PanelPosition, SplitNode};
use txv_widgets::tiled_workspace::TiledWorkspace;

pub use txv_widgets::tiled_workspace::types::LayoutMode;

mod dispatch;
mod view_impl;

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

/// The desktop — TiledWorkspace with kairn-specific chrome.
pub struct Desktop {
    workspace: TiledWorkspace,
}

impl Default for Desktop {
    fn default() -> Self {
        Self::new()
    }
}

impl Desktop {
    pub fn new() -> Self {
        let mut ws = Self::create_workspace();
        for i in 0..PANEL_COUNT {
            if let Some(panel) = ws.panel_mut(i) {
                panel.bar_mut().set_handle_keys(false);
            }
        }
        ws.focus_panel(0);
        ws.set_hidden(3, true);
        Self { workspace: ws }
    }

    fn create_workspace() -> TiledWorkspace {
        let configs = vec![
            PanelConfig::fixed("Files", PanelPosition::Left),
            PanelConfig::new("Editor", PanelPosition::Center),
            PanelConfig::new("Tools", PanelPosition::Right),
            PanelConfig::new("Bottom", PanelPosition::Bottom),
        ];
        let wide_layout = SplitNode::v(vec![
            (
                0.7,
                SplitNode::h(vec![
                    (0.2, SplitNode::leaf(0)),
                    (0.4, SplitNode::leaf(1)),
                    (0.4, SplitNode::leaf(2)),
                ]),
            ),
            (0.3, SplitNode::leaf(3)),
        ]);
        let narrow_layout = SplitNode::v(vec![
            (
                0.6,
                SplitNode::h(vec![(0.2, SplitNode::leaf(0)), (0.8, SplitNode::leaf(1))]),
            ),
            (0.4, SplitNode::leaf(2)),
        ]);
        let mut ws = TiledWorkspace::new(configs, wide_layout, narrow_layout, 300);
        ws.set_narrow_threshold(200);
        ws.set_handle_keys(false);
        ws.set_v_divider_gaps(false);
        ws
    }

    /// Access the underlying TiledWorkspace.
    pub fn workspace(&self) -> &TiledWorkspace {
        &self.workspace
    }

    /// Mutable access to the underlying TiledWorkspace.
    pub fn workspace_mut(&mut self) -> &mut TiledWorkspace {
        &mut self.workspace
    }

    pub fn panel(&self, slot: SlotId) -> &TabPanel {
        match self.workspace.panel(slot as usize) {
            Some(p) => p,
            None => unreachable!(),
        }
    }

    pub fn panel_mut(&mut self, slot: SlotId) -> &mut TabPanel {
        match self.workspace.panel_mut(slot as usize) {
            Some(p) => p,
            None => unreachable!(),
        }
    }

    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, view: Box<dyn View>) {
        let id = slot as usize;
        self.workspace.insert_tab(id, title, view);
        if self.workspace.is_hidden(id) {
            self.workspace.set_hidden(id, false);
        }
    }

    pub fn focused_slot(&self) -> SlotId {
        Self::slot_from(self.workspace.focused_panel())
    }

    pub fn focus_slot(&mut self, id: SlotId) {
        self.workspace.focus_panel(id as usize);
    }

    fn slot_from(idx: usize) -> SlotId {
        match idx {
            0 => SlotId::Left,
            1 => SlotId::Center,
            2 => SlotId::Right,
            _ => SlotId::Bottom,
        }
    }

    pub fn active_tab_title(&self, slot: SlotId) -> Option<&str> {
        self.panel(slot).active_title()
    }

    #[allow(deprecated)]
    pub fn close_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.panel_mut(slot).close_tab_by_title(title)
    }

    pub fn tab_count(&self, slot: SlotId) -> usize {
        self.panel(slot).tab_count()
    }

    pub fn set_active_tab(&mut self, slot: SlotId, index: usize) {
        self.panel_mut(slot).set_active(index);
    }

    #[allow(deprecated)]
    pub fn focus_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.panel_mut(slot).focus_tab_by_title(title)
    }

    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut (dyn View + '_)> {
        self.panel_mut(slot).active_view_mut()
    }

    pub fn find_view_mut<T: View + 'static>(&mut self, slot: SlotId) -> Option<&mut T> {
        let panel = self.workspace.panel_mut(slot as usize)?;
        let count = panel.tab_count();
        let idx = (0..count).find(|&i| {
            panel
                .view_at_mut(i)
                .and_then(|v| v.as_any_mut())
                .is_some_and(|a| a.downcast_ref::<T>().is_some())
        })?;
        let panel = self.workspace.panel_mut(slot as usize)?;
        let view = panel.view_at_mut(idx)?;
        view.as_any_mut()?.downcast_mut::<T>()
    }

    pub fn focus_view_mut<T: View + 'static>(&mut self, slot: SlotId) -> Option<&mut T> {
        let panel = self.workspace.panel_mut(slot as usize)?;
        let count = panel.tab_count();
        let idx = (0..count).find(|&i| {
            panel
                .view_at_mut(i)
                .and_then(|v| v.as_any_mut())
                .is_some_and(|a| a.downcast_ref::<T>().is_some())
        })?;
        let panel = self.workspace.panel_mut(slot as usize)?;
        panel.set_active(idx);
        let view = panel.active_view_mut()?;
        view.as_any_mut()?.downcast_mut::<T>()
    }

    pub fn next_tab_name(&self, slot: SlotId, prefix: &str) -> String {
        self.panel(slot).next_tab_name(prefix)
    }

    pub fn rename_focused_tab(&mut self, new_user_part: &str) {
        let slot = self.focused_slot();
        self.panel_mut(slot).rename_user_part(new_user_part);
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.workspace.set_layout_mode(mode);
    }

    pub fn layout_mode(&self) -> LayoutMode {
        self.workspace.layout_mode()
    }

    pub fn set_wide_threshold(&mut self, threshold: u16) {
        self.workspace.set_wide_threshold(threshold);
    }

    pub fn is_tall(&self) -> bool {
        !self.workspace.is_wide()
    }

    pub fn layout_rects(&self) -> [Rect; PANEL_COUNT] {
        let mut rects = [Rect::default(); PANEL_COUNT];
        for (i, rect) in rects.iter_mut().enumerate() {
            if let Some(child) = self.workspace.child(i) {
                *rect = child.bounds();
            }
        }
        rects
    }
}
