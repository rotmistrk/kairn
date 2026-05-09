//! SlottedDesktop — tiled layout with 4 named slots, each with tabs.
//!
//! ```text
//! ┌──────┬────────────────────┬──────────┐
//! │ left │ center             │ right    │
//! │[tabs]│ [tabs]             │ [tabs]   │
//! ├──────┴────────────────────┴──────────┤
//! │ bottom [tabs]                        │
//! └──────────────────────────────────────┘
//! ```
//!
//! The desktop knows NOTHING about what views are inside each slot.
//! It holds `Box<dyn View>` and dispatches events to the focused slot.

use txv::layout::Rect;
use txv::surface::Surface;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};
use txv_widgets::{TabBar, TabEntry};

use crate::commands::*;
use crate::types::SlotId;

/// A single slot containing tabbed views.
struct Slot {
    tabs: Vec<Box<dyn View>>,
    active: usize,
    visible: bool,
    /// Size in columns (left/right) or rows (bottom).
    size: u16,
    tab_bar: TabBar,
}

impl Slot {
    fn new(size: u16) -> Self {
        Self {
            tabs: Vec::new(),
            active: 0,
            visible: true,
            size,
            tab_bar: TabBar::new(),
        }
    }

    fn active_view(&self) -> Option<&dyn View> {
        self.tabs.get(self.active).map(|v| v.as_ref())
    }

    fn active_view_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.tabs.get_mut(self.active)
    }

    fn add_tab(&mut self, title: &str, view: Box<dyn View>) {
        self.tab_bar.add(TabEntry {
            title: title.to_string(),
            modified: false,
        });
        self.tabs.push(view);
        self.active = self.tabs.len() - 1;
        self.tab_bar.set_active(self.active);
    }

    fn close_active(&mut self) -> Option<Box<dyn View>> {
        if self.tabs.is_empty() {
            return None;
        }
        let removed = self.tabs.remove(self.active);
        self.tab_bar.remove(self.active);
        if self.active >= self.tabs.len() && !self.tabs.is_empty() {
            self.active = self.tabs.len() - 1;
        }
        self.tab_bar.set_active(self.active);
        Some(removed)
    }

    fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active = (self.active + 1) % self.tabs.len();
            self.tab_bar.set_active(self.active);
        }
    }

    fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
            self.tab_bar.set_active(self.active);
        }
    }

    fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }

    fn has_tabs(&self) -> bool {
        self.tabs.len() > 1
    }
}

/// The slotted desktop layout.
pub struct SlottedDesktop {
    slots: [Slot; 4],
    focused: SlotId,
    zoomed: Option<SlotId>,
    bounds: Rect,
}

impl SlottedDesktop {
    /// Create a new desktop with default slot sizes.
    pub fn new() -> Self {
        Self {
            slots: [
                Slot::new(25),  // left
                Slot::new(0),   // center (fill)
                Slot::new(40),  // right
                Slot::new(10),  // bottom
            ],
            focused: SlotId::Center,
            zoomed: None,
            bounds: Rect { x: 0, y: 0, w: 0, h: 0 },
        }
    }

    /// Insert a view as a new tab in the given slot.
    pub fn insert_view(&mut self, slot: SlotId, title: &str, view: Box<dyn View>) {
        self.slot_mut(slot).add_tab(title, view);
    }

    fn slot(&self, id: SlotId) -> &Slot {
        &self.slots[id as usize]
    }

    fn slot_mut(&mut self, id: SlotId) -> &mut Slot {
        &mut self.slots[id as usize]
    }

    fn focused_slot_mut(&mut self) -> &mut Slot {
        &mut self.slots[self.focused as usize]
    }

    /// Compute layout rects for the 4 slots.
    /// Returns (left, center, right, bottom) rects.
    fn compute_rects(&self) -> [Rect; 4] {
        let b = self.bounds;
        if b.w == 0 || b.h == 0 {
            return [Rect { x: 0, y: 0, w: 0, h: 0 }; 4];
        }

        // If zoomed, the zoomed slot gets everything.
        if let Some(z) = self.zoomed {
            let mut rects = [Rect { x: 0, y: 0, w: 0, h: 0 }; 4];
            rects[z as usize] = b;
            return rects;
        }

        let bottom_slot = &self.slots[SlotId::Bottom as usize];
        let bottom_h = if bottom_slot.visible && !bottom_slot.is_empty() {
            bottom_slot.size.min(b.h.saturating_sub(4))
        } else {
            0
        };
        let top_h = b.h.saturating_sub(bottom_h);

        // Horizontal split of the top area
        let left_slot = &self.slots[SlotId::Left as usize];
        let right_slot = &self.slots[SlotId::Right as usize];

        let left_w = if left_slot.visible && !left_slot.is_empty() {
            left_slot.size.min(b.w.saturating_sub(10))
        } else {
            0
        };
        let right_w = if right_slot.visible && !right_slot.is_empty() {
            right_slot.size.min(b.w.saturating_sub(left_w + 10))
        } else {
            0
        };
        let center_w = b.w.saturating_sub(left_w + right_w);

        [
            Rect { x: b.x, y: b.y, w: left_w, h: top_h },
            Rect { x: b.x + left_w, y: b.y, w: center_w, h: top_h },
            Rect { x: b.x + left_w + center_w, y: b.y, w: right_w, h: top_h },
            Rect { x: b.x, y: b.y + top_h, w: b.w, h: bottom_h },
        ]
    }

    fn focus_slot(&mut self, id: SlotId) -> HandleResult {
        if !self.slot(id).is_empty() {
            self.focused = id;
            HandleResult::Consumed
        } else {
            HandleResult::Ignored
        }
    }

    fn focus_next_slot(&mut self) -> HandleResult {
        let order = [SlotId::Left, SlotId::Center, SlotId::Right, SlotId::Bottom];
        let cur = order.iter().position(|&s| s == self.focused).unwrap_or(0);
        for i in 1..order.len() {
            let next = order[(cur + i) % order.len()];
            if !self.slot(next).is_empty() {
                self.focused = next;
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_command(&mut self, cmd: u16) -> HandleResult {
        match cmd {
            CM_FOCUS_LEFT => self.focus_slot(SlotId::Left),
            CM_FOCUS_CENTER => self.focus_slot(SlotId::Center),
            CM_FOCUS_RIGHT => self.focus_slot(SlotId::Right),
            CM_FOCUS_BOTTOM => self.focus_slot(SlotId::Bottom),
            CM_FOCUS_NEXT_SLOT => self.focus_next_slot(),
            CM_TAB_NEXT => {
                self.focused_slot_mut().next_tab();
                HandleResult::Consumed
            }
            CM_TAB_PREV => {
                self.focused_slot_mut().prev_tab();
                HandleResult::Consumed
            }
            CM_TAB_CLOSE => {
                self.focused_slot_mut().close_active();
                HandleResult::Consumed
            }
            CM_SLOT_GROW => {
                let slot = self.focused_slot_mut();
                slot.size = slot.size.saturating_add(2);
                HandleResult::Consumed
            }
            CM_SLOT_SHRINK => {
                let slot = self.focused_slot_mut();
                slot.size = slot.size.saturating_sub(2).max(5);
                HandleResult::Consumed
            }
            CM_ZOOM_TOGGLE => {
                self.zoomed = if self.zoomed.is_some() {
                    None
                } else {
                    Some(self.focused)
                };
                HandleResult::Consumed
            }
            CM_TOGGLE_LEFT => {
                let s = &mut self.slots[SlotId::Left as usize];
                s.visible = !s.visible;
                HandleResult::Consumed
            }
            CM_TOGGLE_RIGHT => {
                let s = &mut self.slots[SlotId::Right as usize];
                s.visible = !s.visible;
                HandleResult::Consumed
            }
            CM_TOGGLE_BOTTOM => {
                let s = &mut self.slots[SlotId::Bottom as usize];
                s.visible = !s.visible;
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn draw_slot(
        slot: &Slot,
        surface: &mut Surface<'_>,
        ctx: &DrawContext,
        is_focused: bool,
    ) {
        let h = surface.height();
        if h == 0 || surface.width() == 0 {
            return;
        }

        // Draw tab bar if multiple tabs
        let content_y = if slot.has_tabs() { 1 } else { 0 };
        if slot.has_tabs() && h > 1 {
            let mut tab_sub = surface.sub(0, 0, surface.width(), 1);
            slot.tab_bar.draw(&mut tab_sub, ctx);
        }

        // Draw active view
        if let Some(view) = slot.active_view() {
            let content_h = h.saturating_sub(content_y);
            if content_h > 0 {
                let mut view_sub = surface.sub(0, content_y, surface.width(), content_h);
                view.draw(&mut view_sub, ctx);
            }
        }

        // Draw focus border indicator (top-left corner)
        if is_focused {
            let style = txv::cell::Style {
                fg: txv::cell::Color::Ansi(14), // cyan
                ..txv::cell::Style::default()
            };
            surface.put(0, 0, '▌', style);
        }
    }
}

impl View for SlottedDesktop {
    fn draw(&self, surface: &mut Surface<'_>, ctx: &DrawContext) {
        let rects = self.compute_rects();
        let b = self.bounds;

        for (i, rect) in rects.iter().enumerate() {
            if rect.w == 0 || rect.h == 0 {
                continue;
            }
            let rel_x = rect.x.saturating_sub(b.x);
            let rel_y = rect.y.saturating_sub(b.y);
            let mut sub = surface.sub(rel_x, rel_y, rect.w, rect.h);
            let slot_id = [SlotId::Left, SlotId::Center, SlotId::Right, SlotId::Bottom][i];
            Self::draw_slot(&self.slots[i], &mut sub, ctx, self.focused == slot_id);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // If zoomed, only the zoomed slot gets events (except commands)
        if let Some(z) = self.zoomed {
            if let Event::Command(cmd) = event {
                let r = self.handle_command(*cmd);
                if r == HandleResult::Consumed {
                    return r;
                }
            }
            if let Some(view) = self.slots[z as usize].active_view_mut() {
                return view.handle(event);
            }
            return HandleResult::Ignored;
        }

        // Commands handled by desktop itself
        if let Event::Command(cmd) = event {
            let r = self.handle_command(*cmd);
            if r == HandleResult::Consumed {
                return r;
            }
        }

        // Dispatch to focused slot's active view
        if let Some(view) = self.focused_slot_mut().active_view_mut() {
            return view.handle(event);
        }
        HandleResult::Ignored
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
    }

    fn focusable(&self) -> bool {
        true
    }
}
