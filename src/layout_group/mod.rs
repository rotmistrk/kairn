//! LayoutGroup — the desktop. Holds TabGroup panels, computes layout in set_bounds.
//!
//! set_bounds is the SINGLE source of truth for child bounds.
//! Resize/zoom change constraints then call set_bounds.

use txv_core::prelude::*;
use txv_widgets::TabGroup;

mod layout;

pub use crate::desktop::SlotId;

const WIDE_THRESHOLD: u16 = 200;
const PANEL_COUNT: usize = 4;

/// Layout mode for the desktop.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LayoutMode {
    Auto,
    Wide,
    Tall,
}

/// The desktop — holds 4 TabGroup panels and computes layout.
pub struct LayoutGroup {
    state: ViewState,
    pub panels: [TabGroup; PANEL_COUNT],
    pub focused: usize,
    pub zoomed: Option<usize>,
    pub layout_mode: LayoutMode,
    pub left_width: u16,
    pub right_width: u16,
    pub right_height: u16,
    pub bottom_height: u16,
}

impl LayoutGroup {
    pub fn new() -> Self {
        Self {
            state: ViewState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            panels: [TabGroup::new(), TabGroup::new(), TabGroup::new(), TabGroup::new()],
            focused: SlotId::Left as usize,
            zoomed: None,
            layout_mode: LayoutMode::Auto,
            left_width: 24,
            right_width: 40,
            right_height: 10,
            bottom_height: 10,
        }
    }

    // ─── Public API (mirrors SlottedDesktop) ───────────────────

    pub fn insert_tab(&mut self, slot: SlotId, title: impl Into<String>, view: Box<dyn View>) {
        self.panels[slot as usize].insert_tab(title, view);
        self.recompute_bounds();
    }

    pub fn active_tab_title(&self, slot: SlotId) -> Option<&str> {
        self.panels[slot as usize].active_title()
    }

    pub fn close_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.panels[slot as usize].close_tab_by_title(title)
    }

    pub fn tab_count(&self, slot: SlotId) -> usize {
        self.panels[slot as usize].tab_count()
    }

    pub fn set_active_tab(&mut self, slot: SlotId, index: usize) {
        self.panels[slot as usize].set_active(index);
    }

    pub fn focus_tab_by_title(&mut self, slot: SlotId, title: &str) -> bool {
        self.panels[slot as usize].focus_tab_by_title(title)
    }

    pub fn active_view_mut(&mut self, slot: SlotId) -> Option<&mut Box<dyn View>> {
        self.panels[slot as usize].active_view_mut()
    }

    pub fn focused_slot(&self) -> SlotId {
        match self.focused {
            0 => SlotId::Left,
            1 => SlotId::Center,
            2 => SlotId::Right,
            _ => SlotId::Bottom,
        }
    }

    pub fn focus_slot(&mut self, id: SlotId) {
        let new = id as usize;
        if new == self.focused {
            return;
        }
        self.panels[self.focused].unselect();
        self.focused = new;
        self.panels[self.focused].select();
        self.state.dirty = true;
    }

    pub fn focus_tab(&mut self, slot: SlotId, tab: usize) {
        self.focus_slot(slot);
        self.panels[slot as usize].set_active(tab);
    }

    pub fn toggle_zoom(&mut self) {
        self.zoomed = if self.zoomed.is_some() {
            None
        } else {
            Some(self.focused)
        };
        self.recompute_bounds();
    }

    pub fn cycle_focus(&mut self, dir: i32) {
        let visible: Vec<usize> = (0..PANEL_COUNT).filter(|&i| self.panels[i].tab_count() > 0).collect();
        if visible.is_empty() {
            return;
        }
        let cur = visible.iter().position(|&i| i == self.focused).unwrap_or(0);
        let next = if dir > 0 {
            (cur + 1) % visible.len()
        } else {
            (cur + visible.len() - 1) % visible.len()
        };
        let new_slot = match visible[next] {
            0 => SlotId::Left,
            1 => SlotId::Center,
            2 => SlotId::Right,
            _ => SlotId::Bottom,
        };
        self.focus_slot(new_slot);
    }

    pub fn is_tall(&self) -> bool {
        match self.layout_mode {
            LayoutMode::Wide => false,
            LayoutMode::Tall => true,
            LayoutMode::Auto => self.state.bounds.w < WIDE_THRESHOLD,
        }
    }

    fn recompute_bounds(&mut self) {
        let b = self.state.bounds;
        self.apply_layout(b);
    }
}

impl Default for LayoutGroup {
    fn default() -> Self {
        Self::new()
    }
}

impl View for LayoutGroup {
    fn bounds(&self) -> Rect {
        self.state.bounds
    }
    fn set_bounds(&mut self, r: Rect) {
        self.state.bounds = r;
        self.apply_layout(r);
        self.state.dirty = true;
    }
    fn options(&self) -> ViewOptions {
        self.state.options
    }
    fn title(&self) -> &str {
        ""
    }
    fn needs_redraw(&self) -> bool {
        self.state.dirty || self.panels.iter().any(|p| p.needs_redraw())
    }
    fn mark_redrawn(&mut self) {
        self.state.dirty = false;
        for p in &mut self.panels {
            p.mark_redrawn();
        }
    }
    fn select(&mut self) {
        self.state.focused = true;
        self.panels[self.focused].select();
    }
    fn unselect(&mut self) {
        self.state.focused = false;
        self.panels[self.focused].unselect();
    }
    fn draw(&self, surface: &mut Surface) {
        for panel in &self.panels {
            if panel.bounds().w > 0 && panel.bounds().h > 0 {
                panel.draw(surface);
            }
        }
    }
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        self.panels[self.focused].handle(event, queue)
    }
}
