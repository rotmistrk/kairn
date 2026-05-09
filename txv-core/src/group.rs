//! Group — a View that owns and dispatches to child Views.
//!
//! Three-phase dispatch:
//! 1. Preprocess: children with `options().preprocess` see event first
//! 2. Focused/modal: the modal child (if any) or focused child handles
//! 3. Postprocess: children with `options().postprocess` see event last

use crate::event::Event;
use crate::view::{EventQueue, HandleResult, View, ViewOptions, ViewState};

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
        self.view.dirty = true;
    }

    pub fn remove(&mut self, index: usize) -> Box<dyn View> {
        let child = self.children.remove(index);
        if self.focused >= self.children.len() && self.focused > 0 {
            self.focused -= 1;
        }
        self.view.dirty = true;
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
                return; // no focusable child
            }
        }
        if old != next {
            self.children[old].unselect();
            self.focused = next;
            self.children[next].select();
            self.view.dirty = true;
        }
    }

    pub fn focus_prev(&mut self) {
        if self.children.is_empty() {
            return;
        }
        let old = self.focused;
        let count = self.children.len();
        let mut prev = if old == 0 { count - 1 } else { old - 1 };
        let start = prev;
        loop {
            if self.children[prev].options().focusable {
                break;
            }
            prev = if prev == 0 { count - 1 } else { prev - 1 };
            if prev == start {
                return;
            }
        }
        if old != prev {
            self.children[old].unselect();
            self.focused = prev;
            self.children[prev].select();
            self.view.dirty = true;
        }
    }

    /// Three-phase event dispatch.
    pub fn dispatch(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
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
            if child.options().postprocess
                && child.handle(event, queue) == HandleResult::Consumed
            {
                return HandleResult::Consumed;
            }
        }

        HandleResult::Ignored
    }

    /// Returns true if any child needs redraw.
    pub fn any_dirty(&self) -> bool {
        self.view.dirty
            || self.children.iter().any(|c| c.needs_redraw())
    }

    fn modal_child(&self) -> Option<usize> {
        self.children.iter().position(|c| c.options().modal)
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

/// Delegates View trait methods for a group (ViewState via `$field.view` + group methods).
///
/// Usage: `delegate_group_state!(group);` inside `impl View for MyGroup { ... }`
/// You still implement `draw()` and `handle()` yourself.
///
/// Override usage (same as delegate_view_state!):
/// ```rust,ignore
/// delegate_group_state!(group, override { set_bounds, needs_redraw });
/// ```
#[macro_export]
macro_rules! delegate_group_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.view.bounds
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.view.bounds = r;
            self.$field.view.dirty = true;
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.view.options
        }
        fn title(&self) -> &str {
            &self.$field.view.title
        }
        fn needs_redraw(&self) -> bool {
            self.$field.any_dirty()
        }
        fn mark_redrawn(&mut self) {
            self.$field.view.dirty = false;
            for child in &mut self.$field.children {
                child.mark_redrawn();
            }
        }
        fn select(&mut self) {
            self.$field.view.focused = true;
            self.$field.view.dirty = true;
            if let Some(child) = self.$field.children.get_mut(self.$field.focused) {
                child.select();
            }
        }
        fn unselect(&mut self) {
            self.$field.view.focused = false;
            self.$field.view.dirty = true;
            if let Some(child) = self.$field.children.get_mut(self.$field.focused) {
                child.unselect();
            }
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.view.bounds
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.view.bounds = r;
                self.$field.view.dirty = true;
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.view.options
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                &self.$field.view.title
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.any_dirty()
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.view.dirty = false;
                for child in &mut self.$field.children {
                    child.mark_redrawn();
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.view.focused = true;
                self.$field.view.dirty = true;
                if let Some(child) = self.$field.children.get_mut(self.$field.focused) {
                    child.select();
                }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.view.focused = false;
                self.$field.view.dirty = true;
                if let Some(child) = self.$field.children.get_mut(self.$field.focused) {
                    child.unselect();
                }
            }
        });
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::surface::Surface;

    struct DummyView {
        state: ViewState,
    }

    impl DummyView {
        fn new(focusable: bool) -> Self {
            Self {
                state: ViewState::new(ViewOptions {
                    focusable,
                    ..ViewOptions::default()
                }),
            }
        }
    }

    impl View for DummyView {
        crate::delegate_view_state!(state);

        fn draw(&self, _surface: &mut Surface) {}

        fn handle(
            &mut self,
            _event: &Event,
            _queue: &mut EventQueue,
        ) -> HandleResult {
            HandleResult::Ignored
        }
    }

    #[test]
    fn focus_next_skips_unfocusable() {
        let mut g = GroupState::default();
        g.insert(Box::new(DummyView::new(true)));
        g.insert(Box::new(DummyView::new(false)));
        g.insert(Box::new(DummyView::new(true)));
        g.children[0].select();
        g.focus_next();
        assert_eq!(g.focused, 2);
    }

    #[test]
    fn focus_prev_wraps() {
        let mut g = GroupState::default();
        g.insert(Box::new(DummyView::new(true)));
        g.insert(Box::new(DummyView::new(true)));
        g.insert(Box::new(DummyView::new(true)));
        g.children[0].select();
        g.focus_prev();
        assert_eq!(g.focused, 2);
    }

    #[test]
    fn three_phase_dispatch() {
        use crate::event::{KeyCode, KeyEvent, KeyMod};

        struct PreView {
            state: ViewState,
        }
        impl View for PreView {
            crate::delegate_view_state!(state);
            fn draw(&self, _s: &mut Surface) {}
            fn handle(
                &mut self,
                _event: &Event,
                _queue: &mut EventQueue,
            ) -> HandleResult {
                HandleResult::Consumed
            }
        }

        let mut g = GroupState::default();
        g.insert(Box::new(PreView {
            state: ViewState::new(ViewOptions {
                preprocess: true,
                focusable: false,
                ..ViewOptions::default()
            }),
        }));
        g.insert(Box::new(DummyView::new(true)));
        g.focused = 1;

        let ev = Event::Key(KeyEvent {
            code: KeyCode::Char('x'),
            modifiers: KeyMod::default(),
        });
        let mut queue = EventQueue::new();
        let result = g.dispatch(&ev, &mut queue);
        assert_eq!(result, HandleResult::Consumed);
    }
}
