//! Group — a View that owns and dispatches to child Views.
//!
//! Three-phase dispatch:
//! 1. Preprocess: children with `options().preprocess` see event first
//! 2. Focused/modal: the modal child (if any) or focused child handles
//! 3. Postprocess: children with `options().postprocess` see event last

mod dispatch;

use crate::view::{View, ViewOptions, ViewState};

/// Common group state — embed in any view that owns children.
pub struct GroupState {
    pub view: ViewState,
    /// Framework-internal: use child(), child_mut(), focused_child_mut() in new code.
    pub children: Vec<Box<dyn View>>,
    /// Framework-internal: use focused_index(), set_focused_index(), switch_focus() in new code.
    pub focused: usize,
}

impl GroupState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            view: ViewState::new(options),
            children: Vec::new(),
            focused: 0,
        }
    }

    pub fn insert(&mut self, child: Box<dyn View>) {
        self.children.push(child);
        self.view.mark_dirty();
    }

    pub fn remove(&mut self, index: usize) -> Box<dyn View> {
        let child = self.children.remove(index);
        if self.focused >= self.children.len() && self.focused > 0 {
            self.focused -= 1;
        }
        self.view.mark_dirty();
        child
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn focused_index(&self) -> usize {
        self.focused
    }

    pub fn set_focused_index(&mut self, index: usize) {
        if index < self.children.len() {
            self.focused = index;
        }
    }

    /// Get immutable reference to a child by index.
    pub fn child(&self, index: usize) -> Option<&dyn View> {
        self.children.get(index).map(|c| c.as_ref())
    }

    /// Get mutable reference to a child by index.
    pub fn child_mut(&mut self, index: usize) -> Option<&mut Box<dyn View>> {
        self.children.get_mut(index)
    }

    /// Get the focused child (immutable).
    pub fn focused_child(&self) -> Option<&dyn View> {
        self.children.get(self.focused).map(|c| c.as_ref())
    }

    /// Get the focused child (mutable).
    pub fn focused_child_mut(&mut self) -> Option<&mut Box<dyn View>> {
        self.children.get_mut(self.focused)
    }

    /// Set bounds on a child by index.
    pub fn set_child_bounds(&mut self, index: usize, rect: crate::geometry::Rect) {
        if let Some(child) = self.children.get_mut(index) {
            child.set_bounds(rect);
        }
    }

    /// Select the focused child, unselect the previous.
    pub fn select_focused(&mut self) {
        if let Some(child) = self.children.get_mut(self.focused) {
            child.select();
        }
    }

    /// Unselect the focused child.
    pub fn unselect_focused(&mut self) {
        if let Some(child) = self.children.get_mut(self.focused) {
            child.unselect();
        }
    }

    /// Switch focus to a new index (unselects old, selects new).
    pub fn switch_focus(&mut self, new_index: usize) {
        if new_index >= self.children.len() || new_index == self.focused {
            return;
        }
        self.children[self.focused].unselect();
        self.focused = new_index;
        self.children[self.focused].select();
        self.view.mark_dirty();
    }

    /// Iterate over children immutably.
    pub fn children_iter(&self) -> impl Iterator<Item = &dyn View> {
        self.children.iter().map(|c| c.as_ref())
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.children.is_empty()
    }

    pub fn focus_next(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let old = self.focused;
        let count = self.children.len();
        let mut next = (old + 1) % count;
        let start = next;
        loop {
            if self.children[next].options().focusable {
                break;
            }
            next = (next + 1) % count;
            if next == start {
                return;
            }
        }
        if old != next {
            self.children[old].unselect();
            self.focused = next;
            self.children[next].select();
            self.view.mark_dirty();
        }
    }

    pub fn focus_prev(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let old = self.focused;
        let count = self.children.len();
        let mut prev = if old == 0 {
            count - 1
        } else {
            old - 1
        };
        let start = prev;
        loop {
            if self.children[prev].options().focusable {
                break;
            }
            prev = if prev == 0 {
                count - 1
            } else {
                prev - 1
            };
            if prev == start {
                return;
            }
        }
        if old != prev {
            self.children[old].unselect();
            self.focused = prev;
            self.children[prev].select();
            self.view.mark_dirty();
        }
    }
}

impl Default for GroupState {
    fn default() -> Self {
        Self::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        })
    }
}

#[cfg(test)]
mod tests;
