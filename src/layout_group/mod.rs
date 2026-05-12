//! LayoutGroup — the desktop. Uses GroupState with 4 TabGroup children.
//!
//! set_bounds is the SINGLE source of truth for child bounds.
//! Resize/zoom change constraints then call set_bounds.

use txv_core::prelude::*;
use txv_widgets::TabGroup;

mod chrome;
mod dispatch;
mod layout;
mod view_impl;

/// Identifies one of the four panel slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotId {
    Left = 0,
    Center = 1,
    Right = 2,
    Bottom = 3,
}

pub(crate) const WIDE_THRESHOLD: u16 = 200;
pub(crate) const TALL_THRESHOLD: u16 = 176;
pub(crate) const PANEL_COUNT: usize = 4;

/// Layout mode for the desktop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutMode {
    Auto,
    Wide,
    Tall,
}

/// The desktop — GroupState with 4 TabGroup children, custom layout.
pub struct LayoutGroup {
    pub(crate) group: GroupState,
    pub zoomed: Option<usize>,
    pub layout_mode: LayoutMode,
    pub left_width: u16,
    pub right_width: u16,
    pub right_height: u16,
    pub bottom_height: u16,
    /// Hysteresis: last known tall/wide state for Auto mode.
    was_tall: bool,
}

impl LayoutGroup {
    pub fn new() -> Self {
        let mut group = GroupState::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        });
        // Insert 4 TabGroup panels as children
        for _ in 0..PANEL_COUNT {
            group.insert(Box::new(TabGroup::new()));
        }
        group.focused = SlotId::Left as usize;
        Self {
            group,
            zoomed: None,
            layout_mode: LayoutMode::Auto,
            left_width: 24,
            right_width: 60,
            right_height: 10,
            bottom_height: 10,
            was_tall: true,
        }
    }

    /// Access a panel as TabGroup (downcast from Box<dyn View>).
    pub fn panel(&self, slot: SlotId) -> &TabGroup {
        let child = self.group.child(slot as usize).expect("valid slot");
        // SAFETY: we only insert TabGroup instances at construction
        unsafe { &*(child as *const dyn View as *const TabGroup) }
    }

    /// Access a panel mutably as TabGroup.
    pub fn panel_mut(&mut self, slot: SlotId) -> &mut TabGroup {
        let child = self.group.child_mut(slot as usize).expect("valid slot");
        // SAFETY: we only insert TabGroup instances at construction
        let ptr: *mut dyn View = &mut **child;
        unsafe { &mut *(ptr as *mut TabGroup) }
    }

    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, view: Box<dyn View>) {
        self.panel_mut(slot).insert_tab(title, view);
        if self.group.view.bounds().w > 0 {
            self.recompute_bounds();
        }
    }

    pub fn active_tab_title(&self, slot: SlotId) -> Option<&str> {
        self.panel(slot).active_title()
    }

    pub fn close_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.panel_mut(slot).close_tab_by_title(title)
    }

    pub fn tab_count(&self, slot: SlotId) -> usize {
        self.panel(slot).tab_count()
    }

    pub fn set_active_tab(&mut self, slot: SlotId, index: usize) {
        self.panel_mut(slot).set_active(index);
    }

    pub fn focus_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.panel_mut(slot).focus_tab_by_title(title)
    }

    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut Box<dyn View>> {
        self.panel_mut(slot).active_view_mut()
    }

    pub fn focused_slot(&self) -> SlotId {
        match self.group.focused_index() {
            0 => SlotId::Left,
            1 => SlotId::Center,
            2 => SlotId::Right,
            _ => SlotId::Bottom,
        }
    }

    pub fn focus_slot(&mut self, id: SlotId) {
        let new = id as usize;
        self.group.switch_focus(new);
    }

    pub fn focus_tab(&mut self, slot: SlotId, tab: usize) {
        self.focus_slot(slot);
        self.panel_mut(slot).set_active(tab);
    }

    pub fn toggle_zoom(&mut self) {
        self.zoomed = if self.zoomed.is_some() {
            None
        } else {
            Some(self.group.focused_index())
        };
        self.recompute_bounds();
    }

    pub fn cycle_focus(&mut self, dir: i32) {
        let visible: Vec<usize> = (0..PANEL_COUNT)
            .filter(|&i| self.panel(Self::slot_from(i)).tab_count() > 0)
            .collect();
        if visible.is_empty() {
            return;
        }
        let cur = visible.iter().position(|&i| i == self.group.focused_index()).unwrap_or(0);
        let next = if dir > 0 {
            (cur + 1) % visible.len()
        } else {
            (cur + visible.len() - 1) % visible.len()
        };
        self.focus_slot(Self::slot_from(visible[next]));
    }

    pub fn is_tall(&self) -> bool {
        match self.layout_mode {
            LayoutMode::Wide => false,
            LayoutMode::Tall => true,
            LayoutMode::Auto => {
                let w = self.group.view.bounds().w;
                if w >= WIDE_THRESHOLD {
                    false
                } else if w <= TALL_THRESHOLD {
                    true
                } else {
                    self.was_tall // hysteresis: stay in current state
                }
            }
        }
    }

    pub fn layout_rects(&self) -> [Rect; PANEL_COUNT] {
        self.compute_rects(self.group.view.bounds())
    }

    pub fn next_tab_name(&self, slot: SlotId, prefix: &str) -> String {
        self.panel(slot).next_tab_name(prefix)
    }

    pub fn rename_focused_tab(&mut self, new_user_part: &str) {
        let focused = self.group.focused_index();
        self.panel_mut(Self::slot_from(focused)).rename_user_part(new_user_part);
    }

    fn recompute_bounds(&mut self) {
        let b = self.group.view.bounds();
        self.apply_layout(b);
    }

    fn slot_from(idx: usize) -> SlotId {
        match idx {
            0 => SlotId::Left,
            1 => SlotId::Center,
            2 => SlotId::Right,
            _ => SlotId::Bottom,
        }
    }
}

impl Default for LayoutGroup {
    fn default() -> Self {
        Self::new()
    }
}
