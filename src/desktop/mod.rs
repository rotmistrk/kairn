//! SlottedDesktop — tiled layout with 4 named slots, each containing tabs.

mod chrome;
mod layout;

use crate::commands::*;
use txv_core::prelude::*;

/// Identifies one of the four slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotId { Left, Center, Right, Bottom }

/// Layout mode for the desktop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutMode { Auto, Wide, Tall }

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
        Self { tabs: Vec::new(), active: 0, visible: true, size }
    }

    fn active_view(&self) -> Option<&dyn View> {
        self.tabs.get(self.active).map(|(_, v)| v.as_ref())
    }

    fn active_view_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.tabs.get_mut(self.active).map(|(_, v)| v)
    }

    fn tab_next(&mut self) {
        if !self.tabs.is_empty() { self.active = (self.active + 1) % self.tabs.len();  }
    }

    fn tab_prev(&mut self) {
        if !self.tabs.is_empty() {
            self.active = if self.active == 0 { self.tabs.len() - 1 } else { self.active - 1 }; 
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
        if bounds.w == 0 || bounds.h == 0 { return; }
        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);
        let i = slot_id as usize;
        let r = if tall && slot_id == SlotId::Right { rects[SlotId::Bottom as usize] } else { rects[i] };
        if let Some(v) = self.slots[i].active_view_mut() {
            v.set_bounds(r);
            if slot_id == self.focused {
                v.select();
            }
        }
    }
}

impl Default for SlottedDesktop {
    fn default() -> Self { Self::new() }
}

impl SlottedDesktop {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions { focusable: true, ..ViewOptions::default() }),
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
        s.tabs.push((title.into(), view));
        s.active = s.tabs.len() - 1; self.group.view.dirty = true;
        s.visible = true;
        self.group.view.dirty = true;
    }

    pub fn focus_tab(&mut self, slot: SlotId, tab: usize) {
        self.focus_slot(slot);
        let s = &mut self.slots[slot as usize];
        if tab < s.tabs.len() {
            s.active = tab; self.group.view.dirty = true;
            self.sync_active_bounds(slot);
            self.group.view.dirty = true;
        }
    }

    pub fn close_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        let s = &mut self.slots[slot as usize];
        if let Some(idx) = s.tabs.iter().position(|(t, _)| t == title) {
            s.tabs.remove(idx);
            if s.active >= s.tabs.len() && s.active > 0 { s.active -= 1; }
            self.group.view.dirty = true;
            return true;
        }
        false
    }

    pub fn tab_count(&self, slot: SlotId) -> usize { self.slots[slot as usize].tabs.len() }

    pub fn focused_slot(&self) -> SlotId { self.focused }

    pub fn layout_rects(&self) -> [Rect; SLOT_COUNT] { self.layout(self.group.view.bounds) }

    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut Box<dyn View>> {
        self.slots[slot as usize].active_view_mut()
    }

    pub fn focus_slot(&mut self, id: SlotId) {
        if id == self.focused { return; }
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() { v.unselect(); }
        self.focused = id;
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() { v.select(); }
        self.group.view.dirty = true;
    }

    fn cycle_focus(&mut self, dir: i32) {
        let visible: Vec<SlotId> = [SlotId::Left, SlotId::Center, SlotId::Right, SlotId::Bottom]
            .iter().copied()
            .filter(|&sid| { let s = &self.slots[sid as usize]; s.visible && !s.tabs.is_empty() })
            .collect();
        if visible.is_empty() { return; }
        let cur = visible.iter().position(|&s| s == self.focused).unwrap_or(0);
        let next = if dir > 0 { (cur + 1) % visible.len() } else { (cur + visible.len() - 1) % visible.len() };
        self.focus_slot(visible[next]);
    }

    fn handle_command(&mut self, id: CommandId, _queue: &mut EventQueue) -> HandleResult {
        match id {
            CM_FOCUS_LEFT => { self.focus_slot(SlotId::Left); HandleResult::Consumed }
            CM_FOCUS_CENTER => { self.focus_slot(SlotId::Center); HandleResult::Consumed }
            CM_FOCUS_RIGHT => { self.focus_slot(SlotId::Right); HandleResult::Consumed }
            CM_FOCUS_BOTTOM => { self.focus_slot(SlotId::Bottom); HandleResult::Consumed }
            CM_FOCUS_PREV => { self.cycle_focus(-1); HandleResult::Consumed }
            CM_FOCUS_NEXT => { self.cycle_focus(1); HandleResult::Consumed }
            CM_ZOOM_TOGGLE => {
                self.zoomed = if self.zoomed.is_some() { None } else { Some(self.focused) };
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_NEXT => {
                if let Some(v) = self.slots[self.focused as usize].active_view_mut() { v.unselect(); }
                self.slots[self.focused as usize].tab_next();
                self.sync_active_bounds(self.focused);
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_PREV => {
                if let Some(v) = self.slots[self.focused as usize].active_view_mut() { v.unselect(); }
                self.slots[self.focused as usize].tab_prev();
                self.sync_active_bounds(self.focused);
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_CLOSE => {
                let s = &mut self.slots[self.focused as usize];
                if !s.tabs.is_empty() {
                    s.tabs.remove(s.active);
                    if s.active >= s.tabs.len() && s.active > 0 { s.active -= 1; }
                    self.group.view.dirty = true;
                }
                HandleResult::Consumed
            }
            CM_TAB_DROPDOWN => {
                if self.dropdown.is_some() {
                    self.dropdown = None;
                } else if self.slots[self.focused as usize].tabs.len() > 1 {
                    self.dropdown_cursor = self.slots[self.focused as usize].active;
                    self.dropdown = Some(self.focused);
                }
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn handle_dropdown_key(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        use txv_core::event::KeyCode;
        let Some(slot_id) = self.dropdown else { return HandleResult::Ignored; };
        let tab_count = self.slots[slot_id as usize].tabs.len();
        match &key.code {
            KeyCode::Esc => { self.dropdown = None; self.group.view.dirty = true; }
            KeyCode::Enter => {
                if let Some(v) = self.slots[slot_id as usize].active_view_mut() { v.unselect(); }
                let s = &mut self.slots[slot_id as usize];
                s.active = self.dropdown_cursor; self.group.view.dirty = true;
                self.dropdown = None;
                self.sync_active_bounds(slot_id);
                self.group.view.dirty = true;
            }
            KeyCode::Up | KeyCode::Down if key.modifiers.ctrl && key.modifiers.shift => {
                // C-S-Up acts as Up, C-S-Down acts as Down in dropdown
                if matches!(key.code, KeyCode::Up) {
                    self.dropdown_cursor = if self.dropdown_cursor == 0 { tab_count - 1 } else { self.dropdown_cursor - 1 };
                } else {
                    self.dropdown_cursor = (self.dropdown_cursor + 1) % tab_count;
                }
                self.group.view.dirty = true;
            }
            KeyCode::Up => {
                self.dropdown_cursor = if self.dropdown_cursor == 0 {
                    tab_count - 1
                } else {
                    self.dropdown_cursor - 1
                };
                self.group.view.dirty = true;
            }
            KeyCode::Down => {
                self.dropdown_cursor = (self.dropdown_cursor + 1) % tab_count;
                self.group.view.dirty = true;
            }
            KeyCode::Char(c) if c.is_ascii_digit() => {
                let idx = (*c as u8 - b'0') as usize;
                if let Some(v) = self.slots[slot_id as usize].active_view_mut() { v.unselect(); }
                let s = &mut self.slots[slot_id as usize];
                if idx < s.tabs.len() {
                    s.active = idx;
                }
                self.dropdown = None;
                self.sync_active_bounds(slot_id);
                self.group.view.dirty = true;
            }
            _ => {}
        }
        HandleResult::Consumed
    }
}

impl View for SlottedDesktop {
    delegate_group_state!(group, override { set_bounds, needs_redraw, select, unselect });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.bounds = r;
        self.group.view.dirty = true;
        let rects = self.layout(r);
        let tall = self.is_tall(r.w);
        for (i, slot) in self.slots.iter_mut().enumerate() {
            let rect = if tall && i == SlotId::Right as usize {
                rects[SlotId::Bottom as usize]
            } else { rects[i] };
            if let Some(v) = slot.active_view_mut() { v.set_bounds(rect); }
        }
    }

    fn needs_redraw(&self) -> bool {
        if self.group.view.dirty { return true; }
        self.slots.iter().any(|s| s.active_view().is_some_and(|v| v.needs_redraw()))
    }

    fn select(&mut self) {
        self.group.view.focused = true;
        self.group.view.dirty = true;
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() { v.select(); }
    }

    fn unselect(&mut self) {
        self.group.view.focused = false;
        self.group.view.dirty = true;
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() { v.unselect(); }
    }

    fn draw(&self, surface: &mut Surface) {
        let bounds = self.group.view.bounds;
        if bounds.w == 0 || bounds.h == 0 { return; }
        self.draw_chrome(surface, bounds);
        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);
        for (i, slot) in self.slots.iter().enumerate() {
            let r = if tall && i == SlotId::Right as usize {
                rects[SlotId::Bottom as usize]
            } else { rects[i] };
            if r.w == 0 || r.h == 0 { continue; }
            if let Some(view) = slot.active_view() { view.draw(surface); }
        }
        self.draw_dropdown(surface, bounds);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Dropdown intercepts all keys when open
        if self.dropdown.is_some() {
            if let Event::Key(key) = event {
                return self.handle_dropdown_key(key);
            }
        }
        if let Event::Command { id, .. } = event {
            let r = self.handle_command(*id, queue);
            if r == HandleResult::Consumed { return r; }
        }
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            let r = v.handle(event, queue);
            if r == HandleResult::Consumed { return r; }
        }
        HandleResult::Ignored
    }
}
