//! Three-phase event dispatch and delegate_group_state! macro.

use crate::event::Event;
use crate::view::{EventQueue, HandleResult};

use super::GroupState;

impl GroupState {
    /// Three-phase event dispatch.
    pub fn dispatch(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Phase 1: preprocess
        for child in &mut self.children {
            if child.options().preprocess {
                log::trace!("Group dispatch: preprocess child");
                if child.handle(event, queue) == HandleResult::Consumed {
                    log::trace!("Group dispatch: preprocess consumed");
                    return HandleResult::Consumed;
                }
            }
        }

        // Phase 2: modal child or focused child
        let target = self.modal_child().unwrap_or(self.focused);
        if let Some(child) = self.children.get_mut(target) {
            if child.handle(event, queue) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }

        // Phase 3: postprocess
        for child in &mut self.children {
            if child.options().postprocess && child.handle(event, queue) == HandleResult::Consumed {
                return HandleResult::Consumed;
            }
        }

        HandleResult::Ignored
    }

    /// Returns true if any child needs redraw.
    pub fn any_dirty(&self) -> bool {
        self.view.dirty || self.children.iter().any(|c| c.needs_redraw())
    }

    fn modal_child(&self) -> Option<usize> {
        self.children.iter().position(|c| c.options().modal)
    }
}

/// Delegates View trait methods for a group (ViewState via `$field.view` + group methods).
#[macro_export]
macro_rules! delegate_group_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect { self.$field.view.bounds }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.view.bounds = r; self.$field.view.dirty = true; }
        fn options(&self) -> $crate::view::ViewOptions { self.$field.view.options }
        fn title(&self) -> &str { &self.$field.view.title }
        fn needs_redraw(&self) -> bool { self.$field.any_dirty() }
        fn mark_redrawn(&mut self) {
            self.$field.view.dirty = false;
            for child in &mut self.$field.children { child.mark_redrawn(); }
        }
        fn select(&mut self) {
            self.$field.view.focused = true; self.$field.view.dirty = true;
            if let Some(child) = self.$field.children.get_mut(self.$field.focused) { child.select(); }
        }
        fn unselect(&mut self) {
            self.$field.view.focused = false; self.$field.view.dirty = true;
            if let Some(child) = self.$field.children.get_mut(self.$field.focused) { child.unselect(); }
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], { fn bounds(&self) -> $crate::geometry::Rect { self.$field.view.bounds } });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], { fn set_bounds(&mut self, r: $crate::geometry::Rect) { self.$field.view.bounds = r; self.$field.view.dirty = true; } });
        $crate::__dvs_maybe!(options, [$($skip),*], { fn options(&self) -> $crate::view::ViewOptions { self.$field.view.options } });
        $crate::__dvs_maybe!(title, [$($skip),*], { fn title(&self) -> &str { &self.$field.view.title } });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], { fn needs_redraw(&self) -> bool { self.$field.any_dirty() } });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.view.dirty = false;
                for child in &mut self.$field.children { child.mark_redrawn(); }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.view.focused = true; self.$field.view.dirty = true;
                if let Some(child) = self.$field.children.get_mut(self.$field.focused) { child.select(); }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.view.focused = false; self.$field.view.dirty = true;
                if let Some(child) = self.$field.children.get_mut(self.$field.focused) { child.unselect(); }
            }
        });
    };
}
