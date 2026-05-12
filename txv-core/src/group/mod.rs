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
    pub children: Vec<Box<dyn View>>,
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
