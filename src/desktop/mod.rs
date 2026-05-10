//! SlottedDesktop — tiled layout with 4 named slots, each containing tabs.

mod chrome;
mod dispatch;
mod dropdown;
mod layout;

use txv_core::prelude::*;

/// Identifies one of the four slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotId {
    Left,
    Center,
    Right,
    Bottom,
}

/// Layout mode for the desktop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutMode {
    Auto,
    Wide,
    Tall,
}

const SLOT_COUNT: usize = 4;
const TOP_SLOTS: [SlotId; 3] = [SlotId::Left, SlotId::Center, SlotId::Right];

/// A single slot holding a stack of tabbed views.
struct Slot {
    tabs: Vec<(String, Box<dyn View>)>,
    active: usize,
    visible: bool,
    size: u16,
}

impl Slot {
    fn new(size: u16) -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            visible: true,
            size,
        }
    }

    fn active_view(&self) -> Option<&dyn View> {
        self.tabs.get(self.active).map(|(_, v)| v.as_ref())
    }

    fn active_view_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.tabs.get_mut(self.active).map(|(_, v)| v)
    }

    fn tab_next(&mut self) {
        if !self.tabs.is_empty() {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }

    fn tab_prev(&mut self) {
        if !self.tabs.is_empty() {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
        }
    }
}

/// Tiled desktop with Left, Center, Right, Bottom slots.
pub struct SlottedDesktop {
    group: GroupState,
    slots: [Slot; SLOT_COUNT],
    focused: SlotId,
    zoomed: Option<SlotId>,
    layout_mode: LayoutMode,
    dropdown: Option<SlotId>,
    dropdown_cursor: usize,
}

impl SlottedDesktop {
    /// After switching tabs, set bounds on the newly active view and select it.
    fn sync_active_bounds(&mut self, slot_id: SlotId) {
        let bounds = self.group.view.bounds;
        if bounds.w == 0 || bounds.h == 0 {
            return;
        }
        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);
        let i = slot_id as usize;
        let r = if tall && slot_id == SlotId::Right {
            rects[SlotId::Bottom as usize]
        } else {
            rects[i]
        };
        if let Some(v) = self.slots[i].active_view_mut() {
            v.set_bounds(r);
            if slot_id == self.focused {
                v.select();
            }
        }
    }
}

impl Default for SlottedDesktop {
    fn default() -> Self {
        Self::new()
    }
}

impl SlottedDesktop {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            slots: [Slot::new(24), Slot::new(0), Slot::new(40), Slot::new(10)],
            focused: SlotId::Left,
            zoomed: None,
            layout_mode: LayoutMode::Auto,
            dropdown: None,
            dropdown_cursor: 0,
        }
    }

    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, mut view: Box<dyn View>) {
        let rects = self.layout(self.group.view.bounds);
        view.set_bounds(rects[slot as usize]);
        let s = &mut self.slots[slot as usize];
        // Evict oldest (first) tab if at capacity
        if s.tabs.len() >= 10 {
            s.tabs.remove(0);
            s.active = s.active.saturating_sub(1);
        }
        s.tabs.push((title.into(), view));
        s.active = s.tabs.len() - 1;
        self.group.view.dirty = true;
        s.visible = true;
        self.group.view.dirty = true;
    }

    pub fn focus_tab(&mut self, slot: SlotId, tab: usize) {
        self.focus_slot(slot);
        let s = &mut self.slots[slot as usize];
        if tab < s.tabs.len() {
            s.active = tab;
            self.group.view.dirty = true;
            self.sync_active_bounds(slot);
            self.group.view.dirty = true;
        }
    }

    pub fn close_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        let s = &mut self.slots[slot as usize];
        if let Some(idx) = s.tabs.iter().position(|(t, _)| t == title) {
            s.tabs.remove(idx);
            if s.active >= s.tabs.len() && s.active > 0 {
                s.active -= 1;
            }
            self.group.view.dirty = true;
            return true;
        }
        false
    }

    pub fn active_tab_title(&self, slot: SlotId) -> Option<&str> {
        let s = &self.slots[slot as usize];
        s.tabs.get(s.active).map(|(t, _)| t.as_str())
    }

    pub fn tab_count(&self, slot: SlotId) -> usize {
        self.slots[slot as usize].tabs.len()
    }

    /// Find first available name like "Shell:0", "Shell:1", etc. for the given slot.
    pub fn next_tab_name(&self, slot: SlotId, prefix: &str) -> String {
        let s = &self.slots[slot as usize];
        for n in 0..10 {
            let candidate = format!("{prefix}:{n}");
            if !s.tabs.iter().any(|(t, _)| t.starts_with(&candidate)) {
                return candidate;
            }
        }
        format!("{prefix}:0")
    }

    pub fn focused_slot(&self) -> SlotId {
        self.focused
    }

    /// Rename the active tab in the focused slot (tool tabs only).
    /// Preserves the type prefix (e.g., "Shell:") and replaces the user part.
    pub fn rename_focused_tab(&mut self, new_user_part: &str) {
        let s = &mut self.slots[self.focused as usize];
        if let Some((title, _)) = s.tabs.get_mut(s.active) {
            if let Some(colon) = title.find(':') {
                let prefix = &title[..=colon];
                *title = format!("{}{}", prefix, new_user_part);
                self.group.view.dirty = true;
            }
        }
    }

    pub fn layout_rects(&self) -> [Rect; SLOT_COUNT] {
        self.layout(self.group.view.bounds)
    }

    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut Box<dyn View>> {
        self.slots[slot as usize].active_view_mut()
    }

    pub fn focus_slot(&mut self, id: SlotId) {
        if id == self.focused {
            return;
        }
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            v.unselect();
        }
        self.focused = id;
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            v.select();
        }
        self.group.view.dirty = true;
    }

    fn cycle_focus(&mut self, dir: i32) {
        let visible: Vec<SlotId> = [SlotId::Left, SlotId::Center, SlotId::Right, SlotId::Bottom]
            .iter()
            .copied()
            .filter(|&sid| {
                let s = &self.slots[sid as usize];
                s.visible && !s.tabs.is_empty()
            })
            .collect();
        if visible.is_empty() {
            return;
        }
        let cur = visible.iter().position(|&s| s == self.focused).unwrap_or(0);
        let next = if dir > 0 {
            (cur + 1) % visible.len()
        } else {
            (cur + visible.len() - 1) % visible.len()
        };
        self.focus_slot(visible[next]);
    }

    /// M-0..9: select tab by index in the focused slot.
    pub(super) fn handle_alt_digit(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        use txv_core::event::KeyCode;
        if key.modifiers.alt && !key.modifiers.ctrl {
            if let KeyCode::Char(c) = &key.code {
                if c.is_ascii_digit() {
                    let idx = (*c as u8 - b'0') as usize;
                    let s = &self.slots[self.focused as usize];
                    if idx < s.tabs.len() {
                        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
                            v.unselect();
                        }
                        self.slots[self.focused as usize].active = idx;
                        self.sync_active_bounds(self.focused);
                        self.group.view.dirty = true;
                    }
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }
}
