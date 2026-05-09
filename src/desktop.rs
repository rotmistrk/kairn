//! SlottedDesktop — tiled layout with 4 named slots, each with tabs.
//!
//! Draws box-drawing chrome: top line with tab names, vertical dividers,
//! bottom divider. Views get clean inner surfaces after chrome.

use txv::cell::{Color, Style};
use txv::layout::Rect;
use txv::surface::Surface;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};

use crate::commands::*;
use crate::types::SlotId;

/// A single slot containing tabbed views.
pub(crate) struct Slot {
    pub(crate) tabs: Vec<(String, Box<dyn View>)>,
    active: usize,
    visible: bool,
    /// Size in columns (left/right) or rows (bottom).
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

    fn active_title(&self) -> &str {
        self.tabs.get(self.active).map(|(t, _)| t.as_str()).unwrap_or("")
    }

    fn add_tab(&mut self, title: &str, view: Box<dyn View>) {
        self.tabs.push((title.to_string(), view));
        self.active = self.tabs.len() - 1;
    }

    fn find_tab_by_title(&self, title: &str) -> Option<usize> {
        self.tabs.iter().position(|(t, _)| t == title)
    }

    fn set_active(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active = idx;
        }
    }

    fn close_active(&mut self) -> Option<Box<dyn View>> {
        if self.tabs.is_empty() {
            return None;
        }
        let (_, view) = self.tabs.remove(self.active);
        if self.active >= self.tabs.len() && !self.tabs.is_empty() {
            self.active = self.tabs.len() - 1;
        }
        Some(view)
    }

    fn next_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active = (self.active + 1) % self.tabs.len();
        }
    }

    fn prev_tab(&mut self) {
        if self.tabs.len() > 1 {
            self.active = if self.active == 0 {
                self.tabs.len() - 1
            } else {
                self.active - 1
            };
        }
    }

    fn is_empty(&self) -> bool {
        self.tabs.is_empty()
    }
}

/// Chrome styles.
struct Chrome;

impl Chrome {
    fn border_style() -> Style {
        Style { fg: Color::Ansi(7), bg: Color::Reset, ..Style::default() }
    }

    fn active_tab_style() -> Style {
        Style {
            fg: Color::Ansi(14), // bright cyan
            bg: Color::Ansi(4),  // dark blue
            attrs: txv::cell::Attrs { bold: true, ..txv::cell::Attrs::default() },
        }
    }

    fn inactive_tab_style() -> Style {
        Style {
            fg: Color::Ansi(15), // white
            bg: Color::Ansi(8),  // dark gray
            ..Style::default()
        }
    }
}

/// The slotted desktop layout.
pub struct SlottedDesktop {
    pub(crate) slots: [Slot; 4],
    focused: SlotId,
    zoomed: Option<SlotId>,
    bounds: Rect,
}

impl SlottedDesktop {
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

    /// Find a tab by title in a slot and switch to it. Returns true if found.
    pub fn switch_to_tab(&mut self, slot: SlotId, title: &str) -> bool {
        if let Some(idx) = self.slot(slot).find_tab_by_title(title) {
            self.slot_mut(slot).set_active(idx);
            true
        } else {
            false
        }
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

    /// Compute content rects for the 4 slots (after chrome).
    /// Row 0 = top chrome line. Content starts at row 1.
    /// If bottom visible: bottom divider + bottom content at end.
    fn compute_content_rects(&self) -> [Rect; 4] {
        let b = self.bounds;
        if b.w < 3 || b.h < 2 {
            return [Rect { x: 0, y: 0, w: 0, h: 0 }; 4];
        }

        if let Some(z) = self.zoomed {
            let mut rects = [Rect { x: 0, y: 0, w: 0, h: 0 }; 4];
            // Zoomed slot gets everything below top chrome line
            rects[z as usize] = Rect { x: b.x, y: b.y + 1, w: b.w, h: b.h.saturating_sub(1) };
            return rects;
        }

        let bottom_slot = &self.slots[SlotId::Bottom as usize];
        let bottom_h = if bottom_slot.visible && !bottom_slot.is_empty() {
            bottom_slot.size.min(b.h.saturating_sub(4))
        } else {
            0
        };
        // If bottom visible, we need 1 row for bottom divider
        let bottom_divider = if bottom_h > 0 { 1 } else { 0 };
        // Top content height: total - top_chrome(1) - bottom_divider - bottom_content
        let top_content_h = b.h.saturating_sub(1 + bottom_divider + bottom_h);

        // Horizontal: divide among visible top slots with dividers between them
        let left_slot = &self.slots[SlotId::Left as usize];
        let right_slot = &self.slots[SlotId::Right as usize];

        let left_visible = left_slot.visible && !left_slot.is_empty();
        let right_visible = right_slot.visible && !right_slot.is_empty();

        // Count dividers between top slots
        let mut divider_count: u16 = 0;
        if left_visible { divider_count += 1; }
        if right_visible { divider_count += 1; }

        let usable_w = b.w.saturating_sub(divider_count);

        let left_w = if left_visible {
            left_slot.size.min(usable_w.saturating_sub(10))
        } else {
            0
        };
        let right_w = if right_visible {
            right_slot.size.min(usable_w.saturating_sub(left_w + 10))
        } else {
            0
        };
        let center_w = usable_w.saturating_sub(left_w + right_w);

        // Compute x positions accounting for dividers
        let left_x = b.x;
        let center_x = b.x + left_w + if left_visible { 1 } else { 0 };
        let right_x = center_x + center_w + if right_visible { 1 } else { 0 };

        let content_y = b.y + 1; // below top chrome line

        let bottom_y = b.y + 1 + top_content_h + bottom_divider;

        [
            Rect { x: left_x, y: content_y, w: left_w, h: top_content_h },
            Rect { x: center_x, y: content_y, w: center_w, h: top_content_h },
            Rect { x: right_x, y: content_y, w: right_w, h: top_content_h },
            Rect { x: b.x, y: bottom_y, w: b.w, h: bottom_h },
        ]
    }

    /// Compute vertical divider x-positions (relative to bounds).
    fn divider_x_positions(&self) -> Vec<u16> {
        if self.zoomed.is_some() {
            return Vec::new();
        }
        let left_slot = &self.slots[SlotId::Left as usize];
        let right_slot = &self.slots[SlotId::Right as usize];
        let left_visible = left_slot.visible && !left_slot.is_empty();
        let right_visible = right_slot.visible && !right_slot.is_empty();

        let mut positions = Vec::new();
        let mut x: u16 = 0;

        if left_visible {
            let lw = left_slot.size.min(self.bounds.w.saturating_sub(12));
            x += lw;
            positions.push(x);
            x += 1; // divider column
        }

        if right_visible {
            let divider_count = positions.len() as u16 + 1; // +1 for right divider
            let usable = self.bounds.w.saturating_sub(divider_count + positions.len() as u16);
            let rw = right_slot.size.min(usable.saturating_sub(10));
            let center_w = usable.saturating_sub(
                if left_visible { left_slot.size.min(self.bounds.w.saturating_sub(12)) } else { 0 } + rw,
            );
            x += center_w;
            positions.push(x);
        }

        positions
    }

    fn draw_top_chrome(&self, surface: &mut Surface<'_>) {
        let w = surface.width();
        let style = Chrome::border_style();

        // Fill row 0 with ─
        surface.hline(0, 0, w, '─', style);

        // Compute divider positions for ┬
        let rects = self.compute_content_rects();
        let left_visible = rects[SlotId::Left as usize].w > 0;
        let right_visible = rects[SlotId::Right as usize].w > 0;

        // Draw tab names for each visible top slot
        let slot_order = [SlotId::Left, SlotId::Center, SlotId::Right];
        for &sid in &slot_order {
            let slot = &self.slots[sid as usize];
            if slot.is_empty() || (!slot.visible && sid != SlotId::Center) {
                continue;
            }
            let rect = rects[sid as usize];
            if rect.w == 0 {
                continue;
            }
            // Tab bar starts at the slot's x position (relative to bounds)
            let start_x = rect.x.saturating_sub(self.bounds.x);
            self.draw_slot_tabs(surface, slot, start_x, rect.w, sid == self.focused);
        }

        // Draw ┬ at divider positions
        if left_visible {
            let lx = rects[SlotId::Left as usize].w;
            if lx < w {
                surface.put(lx, 0, '┬', style);
            }
        }
        if right_visible {
            let rx = rects[SlotId::Right as usize].x.saturating_sub(self.bounds.x).saturating_sub(1);
            if rx < w && rx > 0 {
                surface.put(rx, 0, '┬', style);
            }
        }
    }

    fn draw_slot_tabs(
        &self,
        surface: &mut Surface<'_>,
        slot: &Slot,
        start_x: u16,
        max_w: u16,
        is_focused: bool,
    ) {
        let mut col = start_x;
        let end = start_x + max_w;

        for (i, (title, _)) in slot.tabs.iter().enumerate() {
            let label = format!("({})", title);
            let lw = label.len() as u16;
            if col + lw > end {
                break;
            }
            let style = if i == slot.active {
                if is_focused {
                    Chrome::active_tab_style()
                } else {
                    Chrome::inactive_tab_style()
                }
            } else {
                Chrome::inactive_tab_style()
            };
            surface.print(col, 0, &label, style);
            col += lw;
        }
    }

    fn draw_vertical_dividers(&self, surface: &mut Surface<'_>) {
        let style = Chrome::border_style();
        let rects = self.compute_content_rects();
        let h = surface.height();

        let left_visible = rects[SlotId::Left as usize].w > 0;
        let right_visible = rects[SlotId::Right as usize].w > 0;

        // Left-center divider
        if left_visible {
            let x = rects[SlotId::Left as usize].w;
            if x < surface.width() {
                let vlen = h.saturating_sub(1); // from row 1 to bottom
                surface.vline(x, 1, vlen, '│', style);
            }
        }

        // Center-right divider
        if right_visible {
            let x = rects[SlotId::Right as usize].x.saturating_sub(self.bounds.x).saturating_sub(1);
            if x > 0 && x < surface.width() {
                let vlen = h.saturating_sub(1);
                surface.vline(x, 1, vlen, '│', style);
            }
        }
    }

    fn draw_bottom_divider(&self, surface: &mut Surface<'_>) {
        let bottom_slot = &self.slots[SlotId::Bottom as usize];
        if !bottom_slot.visible || bottom_slot.is_empty() {
            return;
        }

        let rects = self.compute_content_rects();
        let bottom_rect = rects[SlotId::Bottom as usize];
        if bottom_rect.h == 0 {
            return;
        }

        let style = Chrome::border_style();
        let div_y = bottom_rect.y.saturating_sub(self.bounds.y).saturating_sub(1);
        let w = surface.width();

        surface.hline(0, div_y, w, '─', style);

        // Draw ┴ at vertical divider positions
        let left_visible = rects[SlotId::Left as usize].w > 0;
        let right_visible = rects[SlotId::Right as usize].w > 0;

        if left_visible {
            let x = rects[SlotId::Left as usize].w;
            if x < w {
                surface.put(x, div_y, '┴', style);
            }
        }
        if right_visible {
            let x = rects[SlotId::Right as usize].x.saturating_sub(self.bounds.x).saturating_sub(1);
            if x > 0 && x < w {
                surface.put(x, div_y, '┴', style);
            }
        }
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
                self.zoomed = if self.zoomed.is_some() { None } else { Some(self.focused) };
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
}

impl View for SlottedDesktop {
    fn draw(&self, surface: &mut Surface<'_>, ctx: &DrawContext) {
        // 1. Draw chrome
        self.draw_top_chrome(surface);
        self.draw_vertical_dividers(surface);
        self.draw_bottom_divider(surface);

        // 2. Draw each slot's active view into its content rect
        let rects = self.compute_content_rects();
        let b = self.bounds;

        for (i, rect) in rects.iter().enumerate() {
            if rect.w == 0 || rect.h == 0 {
                continue;
            }
            let slot = &self.slots[i];
            if slot.is_empty() {
                continue;
            }
            if let Some(view) = slot.active_view() {
                let rel_x = rect.x.saturating_sub(b.x);
                let rel_y = rect.y.saturating_sub(b.y);
                let mut sub = surface.sub(rel_x, rel_y, rect.w, rect.h);
                view.draw(&mut sub, ctx);
            }
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
