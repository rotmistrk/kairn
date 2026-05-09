//! Dialog — a modal window (framed, centered, captures all input).

use crate::view::ViewOptions;
use crate::window::WindowState;

/// Common dialog state — embed in modal dialog views.
pub struct DialogState {
    pub window: WindowState,
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            window: WindowState::new(ViewOptions {
                focusable: true,
                modal: true,
                ..ViewOptions::default()
            }),
        }
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

/// Delegates View trait methods for a dialog (via WindowState → GroupState → ViewState).
///
/// Usage: `delegate_dialog_state!(dialog);` inside `impl View for MyDialog { ... }`
/// You still implement `draw()` and `handle()` yourself.
///
/// Override usage (same as delegate_view_state!):
/// ```rust,ignore
/// delegate_dialog_state!(dialog, override { set_bounds });
/// ```
#[macro_export]
macro_rules! delegate_dialog_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.window.group.view.bounds
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.window.group.view.bounds = r;
            self.$field.window.group.view.dirty = true;
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.window.group.view.options
        }
        fn title(&self) -> &str {
            &self.$field.window.group.view.title
        }
        fn needs_redraw(&self) -> bool {
            self.$field.window.group.any_dirty()
        }
        fn mark_redrawn(&mut self) {
            self.$field.window.group.view.dirty = false;
            for child in &mut self.$field.window.group.children {
                child.mark_redrawn();
            }
        }
        fn select(&mut self) {
            self.$field.window.group.view.focused = true;
            self.$field.window.group.view.dirty = true;
            if let Some(child) = self
                .$field
                .window
                .group
                .children
                .get_mut(self.$field.window.group.focused)
            {
                child.select();
            }
        }
        fn unselect(&mut self) {
            self.$field.window.group.view.focused = false;
            self.$field.window.group.view.dirty = true;
            if let Some(child) = self
                .$field
                .window
                .group
                .children
                .get_mut(self.$field.window.group.focused)
            {
                child.unselect();
            }
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.window.group.view.bounds
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.window.group.view.bounds = r;
                self.$field.window.group.view.dirty = true;
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.window.group.view.options
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                &self.$field.window.group.view.title
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.window.group.any_dirty()
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.window.group.view.dirty = false;
                for child in &mut self.$field.window.group.children {
                    child.mark_redrawn();
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.window.group.view.focused = true;
                self.$field.window.group.view.dirty = true;
                if let Some(child) = self
                    .$field
                    .window
                    .group
                    .children
                    .get_mut(self.$field.window.group.focused)
                {
                    child.select();
                }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.window.group.view.focused = false;
                self.$field.window.group.view.dirty = true;
                if let Some(child) = self
                    .$field
                    .window
                    .group
                    .children
                    .get_mut(self.$field.window.group.focused)
                {
                    child.unselect();
                }
            }
        });
    };
}
