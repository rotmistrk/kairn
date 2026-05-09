//! SlottedDesktop — tiled layout with 4 named slots, each containing tabs.
//!
//! ```text
//! ─(Tab1)(Tab2)──┬─(File.rs)──┬─(Shell)──
//! │ left         │ center     │ right
//! ───────────────┴────────────┴──────────
//! │ bottom                               │
//! ```

use crate::commands::*;
use txv_core::prelude::*;

/// Identifies one of the four slots.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SlotId {
    Left,
    Center,
    Right,
    Bottom,
}

const SLOT_COUNT: usize = 4;
const TOP_SLOTS: [SlotId; 3] = [SlotId::Left, SlotId::Center, SlotId::Right];

/// A single slot holding a stack of tabbed views.
struct Slot {
    tabs: Vec<(String, Box<dyn View>)>,
    active: usize,
    visible: bool,
    size: u16, // width for top slots, height for bottom
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
            slots: [
                Slot::new(24), // Left
                Slot::new(0),  // Center (fills remaining)
                Slot::new(40), // Right
                Slot::new(10), // Bottom
            ],
            focused: SlotId::Left,
            zoomed: None,
        }
    }

    /// Insert a view into a specific slot.
    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, mut view: Box<dyn View>) {
        // Set bounds on the new view based on current layout
        let rects = self.layout(self.group.view.bounds);
        view.set_bounds(rects[slot as usize]);
        let s = &mut self.slots[slot as usize];
        s.tabs.push((title.into(), view));
        s.active = s.tabs.len() - 1;
        s.visible = true;
        self.group.view.dirty = true;
    }

    /// Focus a specific tab in a specific slot.
    pub fn focus_tab(&mut self, slot: SlotId, tab: usize) {
        self.focus_slot(slot);
        let s = &mut self.slots[slot as usize];
        if tab < s.tabs.len() {
            s.active = tab;
            self.group.view.dirty = true;
        }
    }

    /// Close a tab by title in a given slot. Returns true if found and closed.
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

    /// Get the tab count in a specific slot.
    pub fn tab_count(&self, slot: SlotId) -> usize {
        self.slots[slot as usize].tabs.len()
    }

    /// Get a mutable reference to the active view in a slot (for downcasting).
    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut Box<dyn View>> {
        self.slots[slot as usize].active_view_mut()
    }

    fn focus_slot(&mut self, id: SlotId) {
        if id == self.focused {
            return;
        }
        // Unselect old
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            v.unselect();
        }
        self.focused = id;
        // Select new
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            v.select();
        }
        self.group.view.dirty = true;
    }

    /// Cycle focus among visible non-empty slots. dir: 1 = next, -1 = prev.
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

    /// Compute inner rects for each slot given total bounds.
    fn layout(&self, bounds: Rect) -> [Rect; SLOT_COUNT] {
        let mut rects = [Rect::default(); SLOT_COUNT];
        if bounds.w == 0 || bounds.h == 0 {
            return rects;
        }

        // Row 0 = chrome (tab bar)
        let chrome_h = 1u16;
        let content_y = bounds.y + chrome_h;

        // Bottom slot
        let bottom = &self.slots[SlotId::Bottom as usize];
        let bottom_h = if bottom.visible && !bottom.tabs.is_empty() {
            bottom.size.min(bounds.h.saturating_sub(chrome_h + 2))
        } else {
            0
        };
        let bottom_divider = if bottom_h > 0 {
            1u16
        } else {
            0
        };

        let top_h = bounds
            .h
            .saturating_sub(chrome_h)
            .saturating_sub(bottom_h)
            .saturating_sub(bottom_divider);

        // Top slots widths
        let left = &self.slots[SlotId::Left as usize];
        let right = &self.slots[SlotId::Right as usize];

        let left_w = if left.visible && !left.tabs.is_empty() {
            left.size.min(bounds.w / 3)
        } else {
            0
        };
        let left_div = if left_w > 0 {
            1u16
        } else {
            0
        };

        let right_w = if right.visible && !right.tabs.is_empty() {
            right.size.min(bounds.w / 3)
        } else {
            0
        };
        let right_div = if right_w > 0 {
            1u16
        } else {
            0
        };

        let center_w = bounds
            .w
            .saturating_sub(left_w)
            .saturating_sub(left_div)
            .saturating_sub(right_w)
            .saturating_sub(right_div);

        // If zoomed, the focused slot gets all space
        if let Some(z) = self.zoomed {
            rects[z as usize] = Rect::new(bounds.x, content_y, bounds.w, bounds.h.saturating_sub(chrome_h));
            return rects;
        }

        let mut x = bounds.x;

        // Left
        rects[SlotId::Left as usize] = Rect::new(x, content_y, left_w, top_h);
        x += left_w + left_div;

        // Center
        rects[SlotId::Center as usize] = Rect::new(x, content_y, center_w, top_h);
        x += center_w + right_div;

        // Right
        rects[SlotId::Right as usize] = Rect::new(x, content_y, right_w, top_h);

        // Bottom
        let bottom_y = content_y + top_h + bottom_divider;
        rects[SlotId::Bottom as usize] = Rect::new(bounds.x, bottom_y, bounds.w, bottom_h);

        rects
    }

    fn draw_chrome(&self, surface: &mut Surface, bounds: Rect) {
        if bounds.w == 0 || bounds.h == 0 {
            return;
        }
        let rects = self.layout(bounds);
        let chrome_style = Style {
            fg: Color::Ansi(7),
            bg: Color::Ansi(0),
            attrs: Attrs::default(),
        };
        // Focused slot + active tab: bright cyan on blue
        let focused_tab_style = Style {
            fg: Color::Ansi(14),
            bg: Color::Ansi(4),
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
        };
        // Unfocused slot + active (top) tab: white on dark gray
        let unfocused_active_style = Style {
            fg: Color::Ansi(15),
            bg: Color::Ansi(8),
            attrs: Attrs {
                bold: true,
                ..Attrs::default()
            },
        };
        // Any slot + inactive tab: dim gray
        let inactive_tab_style = Style {
            fg: Color::Ansi(8),
            bg: Color::Ansi(0),
            attrs: Attrs::default(),
        };

        // Top line: horizontal rule with tabs
        let y = bounds.y;
        surface.hline(bounds.x, y, bounds.w, '─', chrome_style);

        // Draw tabs for each visible top slot
        for &sid in &TOP_SLOTS {
            let slot = &self.slots[sid as usize];
            let r = rects[sid as usize];
            if r.w == 0 || slot.tabs.is_empty() {
                continue;
            }
            let mut tx = r.x;
            for (i, (title, _)) in slot.tabs.iter().enumerate() {
                let label = format!("({})", title);
                let style = if i == slot.active {
                    if sid == self.focused {
                        focused_tab_style
                    } else {
                        unfocused_active_style
                    }
                } else {
                    inactive_tab_style
                };
                if tx + label.len() as u16 > r.x + r.w {
                    break;
                }
                surface.print(tx, y, &label, style);
                tx += label.len() as u16;
            }
        }

        // Vertical dividers between top slots
        let left_r = rects[SlotId::Left as usize];
        let right_r = rects[SlotId::Right as usize];
        let center_r = rects[SlotId::Center as usize];

        if left_r.w > 0 && center_r.w > 0 {
            let div_x = left_r.x + left_r.w;
            surface.put(div_x, y, '┬', chrome_style);
            surface.vline(div_x, y + 1, left_r.h, '│', chrome_style);
        }
        if right_r.w > 0 && center_r.w > 0 {
            let div_x = right_r.x.saturating_sub(1);
            surface.put(div_x, y, '┬', chrome_style);
            surface.vline(div_x, y + 1, right_r.h, '│', chrome_style);
        }

        // Bottom divider
        let bottom_r = rects[SlotId::Bottom as usize];
        if bottom_r.h > 0 {
            let div_y = bottom_r.y.saturating_sub(1);
            surface.hline(bounds.x, div_y, bounds.w, '─', chrome_style);
            // Junction chars
            if left_r.w > 0 && center_r.w > 0 {
                let div_x = left_r.x + left_r.w;
                surface.put(div_x, div_y, '┴', chrome_style);
            }
            if right_r.w > 0 && center_r.w > 0 {
                let div_x = right_r.x.saturating_sub(1);
                surface.put(div_x, div_y, '┴', chrome_style);
            }
        }
    }

    fn handle_command(&mut self, id: CommandId, _queue: &mut EventQueue) -> HandleResult {
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
                self.slots[self.focused as usize].tab_next();
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_PREV => {
                self.slots[self.focused as usize].tab_prev();
                self.group.view.dirty = true;
                HandleResult::Consumed
            }
            CM_TAB_CLOSE => {
                let s = &mut self.slots[self.focused as usize];
                if !s.tabs.is_empty() {
                    s.tabs.remove(s.active);
                    if s.active >= s.tabs.len() && s.active > 0 {
                        s.active -= 1;
                    }
                    self.group.view.dirty = true;
                }
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
        // Propagate bounds to slot views
        let rects = self.layout(r);
        for (i, slot) in self.slots.iter_mut().enumerate() {
            if let Some(v) = slot.active_view_mut() {
                v.set_bounds(rects[i]);
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
        for (i, slot) in self.slots.iter().enumerate() {
            let r = rects[i];
            if r.w == 0 || r.h == 0 {
                continue;
            }
            if let Some(view) = slot.active_view() {
                view.draw(surface);
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Handle commands directed at desktop
        if let Event::Command { id, .. } = event {
            let r = self.handle_command(*id, queue);
            if r == HandleResult::Consumed {
                return r;
            }
        }

        // Dispatch to focused slot's active view
        if let Some(v) = self.slots[self.focused as usize].active_view_mut() {
            let r = v.handle(event, queue);
            if r == HandleResult::Consumed {
                return r;
            }
        }

        HandleResult::Ignored
    }
}
