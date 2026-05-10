//! SlottedDesktop event dispatch — command handling, dropdown keys, View impl.

use txv_core::prelude::*;

use super::{SlotId, SlottedDesktop};
use crate::commands::*;

impl SlottedDesktop {
    pub(super) fn handle_command(&mut self, id: CommandId, queue: &mut EventQueue) -> HandleResult {
        match id {
            CM_FOCUS_LEFT => {
                self.focus_slot(SlotId::Left);
                HandleResult::Consumed
            }
            CM_FOCUS_CENTER => {
                self.focus_slot(SlotId::Center);
                HandleResult::Consumed
            }
            CM_FOCUS_RIGHT => {
                self.focus_slot(SlotId::Right);
                HandleResult::Consumed
            }
            CM_FOCUS_BOTTOM => {
                self.focus_slot(SlotId::Bottom);
                HandleResult::Consumed
            }
            CM_FOCUS_PREV => {
                self.cycle_focus(-1);
                HandleResult::Consumed
            }
            CM_FOCUS_NEXT => {
                self.cycle_focus(1);
                HandleResult::Consumed
            }
            CM_ZOOM_TOGGLE => {
                self.zoomed = if self.zoomed.is_some() {
                    None
                } else {
                    Some(self.focused)
                };
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_NEXT => {
                if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
                    v.unselect();
                }
                self.slots[self.focused as usize].tab_next();
                self.sync_active_bounds(self.focused);
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_PREV => {
                if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
                    v.unselect();
                }
                self.slots[self.focused as usize].tab_prev();
                self.sync_active_bounds(self.focused);
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_CLOSE => {
                let s = &mut self.slots[self.focused as usize];
                if !s.tabs.is_empty() {
                    let title = s.tabs[s.active].0.clone();
                    s.tabs.remove(s.active);
                    s.lru.remove(s.active);
                    if s.active >= s.tabs.len() && s.active > 0 {
                        s.active -= 1;
                    }
                    self.group.view.dirty = true;
                    queue.put_command(CM_FILE_CLOSED, Some(Box::new(title)));
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
            CM_PANEL_GROW => {
                self.resize_focused(2);
                HandleResult::Consumed
            }
            CM_PANEL_SHRINK => {
                self.resize_focused(-2);
                HandleResult::Consumed
            }
            CM_PANEL_GROW_V => {
                self.resize_vertical(2);
                HandleResult::Consumed
            }
            CM_PANEL_SHRINK_V => {
                self.resize_vertical(-2);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
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
            } else {
                rects[i]
            };
            if let Some(v) = slot.active_view_mut() {
                v.set_bounds(rect);
            }
        }
    }

    fn needs_redraw(&self) -> bool {
        if self.group.view.dirty {
            return true;
        }
        self.slots
            .iter()
            .any(|s| s.active_view().is_some_and(|v| v.needs_redraw()))
    }

    fn select(&mut self) {
        self.group.view.focused = true;
        self.group.view.dirty = true;
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            v.select();
        }
    }

    fn unselect(&mut self) {
        self.group.view.focused = false;
        self.group.view.dirty = true;
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            v.unselect();
        }
    }

    fn draw(&self, surface: &mut Surface) {
        let bounds = self.group.view.bounds;
        if bounds.w == 0 || bounds.h == 0 {
            return;
        }
        self.draw_chrome(surface, bounds);
        let rects = self.layout(bounds);
        let tall = self.is_tall(bounds.w);
        for (i, slot) in self.slots.iter().enumerate() {
            let r = if tall && i == SlotId::Right as usize {
                rects[SlotId::Bottom as usize]
            } else {
                rects[i]
            };
            if r.w == 0 || r.h == 0 {
                continue;
            }
            if let Some(view) = slot.active_view() {
                view.draw(surface);
            }
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
        // M-0..9: select tab N in focused slot
        if let Event::Key(key) = event {
            if let HandleResult::Consumed = self.handle_alt_digit(key) {
                return HandleResult::Consumed;
            }
        }
        if let Event::Command { id, .. } = event {
            let r = self.handle_command(*id, queue);
            if r == HandleResult::Consumed {
                return r;
            }
        }
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            let r = v.handle(event, queue);
            if r == HandleResult::Consumed {
                return r;
            }
        }
        HandleResult::Ignored
    }
}
