//! TkDesktop — Group that holds script widgets as children.
//!
//! Widgets are inserted as `Box<dyn View>` into GroupState.
//! The name→index map allows script commands to find their widgets.
//! Layout manager computes bounds via `set_bounds()` on each child.

use std::collections::HashMap;

use txv_core::prelude::*;

use crate::layout_mgr::LayoutManager;

/// Group-based desktop holding script widgets.
pub struct TkDesktop {
    group: GroupState,
    /// Widget name → child index in group.
    pub names: HashMap<String, usize>,
    /// Layout manager (pack model).
    pub layout: LayoutManager,
}

impl TkDesktop {
    pub fn new() -> Self {
        Self {
            group: GroupState::new(ViewOptions {
                focusable: true,
                ..ViewOptions::default()
            }),
            names: HashMap::new(),
            layout: LayoutManager::new(),
        }
    }

    /// Insert a widget with a name. Returns the child index.
    pub fn insert_widget(&mut self, name: String, widget: Box<dyn View>) -> usize {
        let idx = self.group.child_count();
        self.group.insert(widget);
        self.names.insert(name, idx);
        idx
    }

    /// Get a child view by name (immutable).
    pub fn get(&self, name: &str) -> Option<&dyn View> {
        let idx = *self.names.get(name)?;
        self.group.child(idx)
    }

    /// Get a child view by name (mutable).
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Box<dyn View>> {
        let idx = *self.names.get(name)?;
        self.group.child_mut(idx)
    }

    /// Set focus to a named widget.
    pub fn focus(&mut self, name: &str) {
        if let Some(&idx) = self.names.get(name) {
            self.group.switch_focus(idx);
        }
    }

    /// Recompute layout and set bounds on all children.
    fn apply_layout(&mut self) {
        let b = self.group.view.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let positions = self.layout.compute(b);
        for (name, rect) in &positions {
            if let Some(&idx) = self.names.get(name) {
                self.group.set_child_bounds(idx, *rect);
            }
        }
    }
}

impl Default for TkDesktop {
    fn default() -> Self {
        Self::new()
    }
}

impl View for TkDesktop {
    delegate_group_state!(group, override { set_bounds });

    fn set_bounds(&mut self, r: Rect) {
        self.group.view.set_bounds(r);
        self.group.view.mark_dirty();
        self.apply_layout();
    }

    fn draw(&mut self) {
        for i in 0..self.group.child_count() {
            if let Some(child) = self.group.child_mut(i) {
                child.draw();
            }
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        self.group.dispatch(event)
    }
}
