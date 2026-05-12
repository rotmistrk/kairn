//! Window — a framed group (title bar, border, optional shadow).

use crate::group::GroupState;
use crate::view::ViewOptions;

/// Frame drawing style.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum FrameStyle {
    #[default]
    Single,
    Double,
    None,
}

/// Common window state — embed in framed views.
pub struct WindowState {
    pub group: GroupState,
    pub frame: FrameStyle,
    pub shadow: bool,
}

impl WindowState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            group: GroupState::new(options),
            frame: FrameStyle::Single,
            shadow: false,
        }
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        })
    }
}

/// Delegates View trait methods for a window (via GroupState inside WindowState).
///
/// Usage: `delegate_window_state!(window);` inside `impl View for MyWindow { ... }`
/// You still implement `draw()` and `handle()` yourself.
///
/// Override usage (same as delegate_view_state!):
/// ```rust,ignore
/// delegate_window_state!(window, override { set_bounds });
/// ```
#[macro_export]
macro_rules! delegate_window_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.group.view.bounds()
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.group.view.set_bounds(r);
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.group.view.options
        }
        fn title(&self) -> &str {
            &self.$field.group.view.title
        }
        fn needs_redraw(&self) -> bool {
            self.$field.group.any_dirty()
        }
        fn mark_redrawn(&mut self) {
            self.$field.group.view.mark_redrawn();
            for child in &mut self.$field.group.children {
                child.mark_redrawn();
            }
        }
        fn select(&mut self) {
            self.$field.group.view.set_focused(true);
            self.$field.group.view.mark_dirty();
            if let Some(child) =
                self.$field.group.children.get_mut(self.$field.group.focused)
            {
                child.select();
            }
        }
        fn unselect(&mut self) {
            self.$field.group.view.set_focused(false);
            self.$field.group.view.mark_dirty();
            if let Some(child) =
                self.$field.group.children.get_mut(self.$field.group.focused)
            {
                child.unselect();
            }
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.group.view.bounds()
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.group.view.set_bounds(r);
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.group.view.options
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                &self.$field.group.view.title
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.group.any_dirty()
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.group.view.mark_redrawn();
                for child in &mut self.$field.group.children {
                    child.mark_redrawn();
                }
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.group.view.set_focused(true);
                self.$field.group.view.mark_dirty();
                if let Some(child) =
                    self.$field.group.children.get_mut(self.$field.group.focused)
                {
                    child.select();
                }
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.group.view.set_focused(false);
                self.$field.group.view.mark_dirty();
                if let Some(child) =
                    self.$field.group.children.get_mut(self.$field.group.focused)
                {
                    child.unselect();
                }
            }
        });
    };
}
