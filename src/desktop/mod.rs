//! Desktop — wraps TiledWorkspace with kairn's SlotId API and chrome drawing.
//!
//! This is the top-level view for kairn's panel layout. It delegates panel
//! management to TiledWorkspace while providing the familiar SlotId-based
//! accessors that all handler code uses.

use std::collections::HashMap;
use std::time::Instant;

use txv_core::prelude::*;
use txv_widgets::tab_panel::TabPanel;
use txv_widgets::tiled_workspace::types::{PanelConfig, PanelPosition, SplitDir, SplitNode};
use txv_widgets::tiled_workspace::TiledWorkspace;

pub use txv_widgets::tiled_workspace::types::LayoutMode;

mod badges;
mod chrome;
mod dispatch;
mod view_impl;

/// Identifies one of the four panel slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SlotId {
    Left = 0,
    Center = 1,
    Right = 2,
    Bottom = 3,
}

/// Activity state for a terminal tab.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabBadge {
    Busy,
    Idle,
    Exited,
}

const PANEL_COUNT: usize = 4;

/// The desktop — TiledWorkspace with kairn-specific chrome and badge tracking.
pub struct Desktop {
    workspace: TiledWorkspace,
    /// Last output timestamp per terminal tab (slot, tab_index).
    last_output: HashMap<(SlotId, usize), Instant>,
    /// Cached badge state per terminal tab.
    badges: HashMap<(SlotId, usize), TabBadge>,
}

impl Default for Desktop {
    fn default() -> Self {
        Self::new()
    }
}

impl Desktop {
    pub fn new() -> Self {
        let mut ws = Self::create_workspace();
        // Disable internal key handling on tab bars
        for i in 0..PANEL_COUNT {
            if let Some(panel) = ws.panel_mut(i) {
                panel.bar_mut().set_handle_keys(false);
            }
        }
        ws.focus_panel(0);
        ws.set_hidden(3, true);

        Self {
            workspace: ws,
            last_output: HashMap::new(),
            badges: HashMap::new(),
        }
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

    /// Access a panel as TabPanel. SlotId is bounded so this always succeeds.
    pub fn panel(&self, slot: SlotId) -> &TabPanel {
        // SAFETY: SlotId enum has exactly 4 variants matching our 4 panels
        match self.workspace.panel(slot as usize) {
            Some(p) => p,
            None => unreachable!(),
        }
    }

    /// Access a panel mutably as TabPanel. SlotId is bounded so this always succeeds.
    pub fn panel_mut(&mut self, slot: SlotId) -> &mut TabPanel {
        match self.workspace.panel_mut(slot as usize) {
            Some(p) => p,
            None => unreachable!(),
        }
    }

    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, view: Box<dyn View>) {
        let id = slot as usize;
        self.workspace.insert_tab(id, title, view);
        // Auto-show panel when it gets tabs
        if self.workspace.is_hidden(id) {
            self.workspace.set_hidden(id, false);
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

    pub fn focused_slot(&self) -> SlotId {
        Self::slot_from(self.workspace.focused_panel())
    }

    pub fn focus_slot(&mut self, id: SlotId) {
        self.workspace.focus_panel(id as usize);
    }

    pub fn focus_tab(&mut self, slot: SlotId, tab: usize) {
        self.focus_slot(slot);
        self.panel_mut(slot).set_active(tab);
    }

    pub fn toggle_zoom(&mut self) {
        self.workspace.toggle_zoom();
    }

    pub fn is_zoomed(&self) -> bool {
        self.workspace.is_zoomed()
    }

    pub fn cycle_focus(&mut self, dir: i32) {
        if dir > 0 {
            self.workspace.focus_next_visible();
        } else {
            self.workspace.focus_prev_visible();
        }
        // If zoomed, follow focus
        if self.workspace.is_zoomed() {
            self.workspace.set_zoomed(Some(self.workspace.focused_panel()));
        }
    }

    pub fn is_tall(&self) -> bool {
        !self.workspace.is_wide()
    }

    pub fn set_layout_mode(&mut self, mode: LayoutMode) {
        self.workspace.set_layout_mode(mode);
    }

    pub fn layout_mode(&self) -> LayoutMode {
        self.workspace.layout_mode()
    }

    pub fn next_tab_name(&self, slot: SlotId, prefix: &str) -> String {
        self.panel(slot).next_tab_name(prefix)
    }

    pub fn rename_focused_tab(&mut self, new_user_part: &str) {
        let slot = self.focused_slot();
        self.panel_mut(slot).rename_user_part(new_user_part);
    }

    pub fn resize_focused(&mut self, delta: i16) {
        self.workspace.resize_panel(SplitDir::Horizontal, delta);
    }

    pub fn resize_vertical(&mut self, delta: i16) {
        self.workspace.resize_panel(SplitDir::Vertical, delta);
    }

    pub fn set_wide_threshold(&mut self, threshold: u16) {
        self.workspace.set_wide_threshold(threshold);
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

    fn slot_from(idx: usize) -> SlotId {
        match idx {
            0 => SlotId::Left,
            1 => SlotId::Center,
            2 => SlotId::Right,
            _ => SlotId::Bottom,
        }
    }

    /// Draw and blit all visible children onto the workspace buffer.
    pub(crate) fn draw_children(&mut self) {
        let my_bounds = self.workspace.bounds();
        for i in 0..PANEL_COUNT {
            if let Some(child) = self.workspace.child_mut(i) {
                let cb = child.bounds();
                if cb.w > 0 && cb.h > 0 {
                    child.draw();
                }
            }
        }
        let buf_ptr = self.workspace.buffer_mut() as *mut txv_core::prelude::Buffer;
        for i in 0..PANEL_COUNT {
            if let Some(child) = self.workspace.child(i) {
                let cb = child.bounds();
                if cb.w > 0 && cb.h > 0 {
                    let dx = cb.x.saturating_sub(my_bounds.x);
                    let dy = cb.y.saturating_sub(my_bounds.y);
                    unsafe { (*buf_ptr).blit(child.buffer(), dx, dy) };
                }
            }
        }
    }
}
