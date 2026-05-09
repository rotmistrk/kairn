//! View trait, ViewState, EventQueue, and the delegate_view_state! macro.

use std::any::Any;

use crate::event::{CommandId, Event};
use crate::geometry::Rect;
use crate::surface::Surface;

/// Options flags for a View.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct ViewOptions {
    pub preprocess: bool,
    pub postprocess: bool,
    pub focusable: bool,
    pub modal: bool,
}

/// Result of handling an event.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HandleResult {
    Consumed,
    Ignored,
}

/// Event queue — views emit commands via `put_command`.
pub struct EventQueue {
    events: Vec<Event>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn put(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn put_command(
        &mut self,
        id: CommandId,
        data: Option<Box<dyn Any + Send>>,
    ) {
        self.events.push(Event::Command { id, data });
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl Default for EventQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// A rectangular UI element.
pub trait View: Send {
    fn draw(&self, surface: &mut Surface);
    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult;
    fn select(&mut self) {}
    fn unselect(&mut self) {}
    fn bounds(&self) -> Rect;
    fn set_bounds(&mut self, rect: Rect);
    fn options(&self) -> ViewOptions {
        ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        }
    }
    fn title(&self) -> &str {
        ""
    }
    fn needs_redraw(&self) -> bool {
        true
    }
    fn mark_redrawn(&mut self) {}
}

/// Common view state — embed in every view.
pub struct ViewState {
    pub bounds: Rect,
    pub options: ViewOptions,
    pub dirty: bool,
    pub focused: bool,
    pub title: String,
}

impl ViewState {
    pub fn new(options: ViewOptions) -> Self {
        Self {
            bounds: Rect::default(),
            options,
            dirty: true,
            focused: false,
            title: String::new(),
        }
    }
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new(ViewOptions {
            focusable: true,
            ..ViewOptions::default()
        })
    }
}

/// Delegates View trait boilerplate to a `ViewState` field.
///
/// # Basic usage
///
/// ```rust,ignore
/// delegate_view_state!(state);
/// ```
///
/// Generates all standard View methods (bounds, set_bounds, options, title,
/// needs_redraw, mark_redrawn, select, unselect).
///
/// # Override usage
///
/// When a widget needs custom behavior for some methods, list the overridden
/// method names in an `override` block. The macro generates everything *except*
/// those methods, which you provide manually in the same `impl` block:
///
/// ```rust,ignore
/// delegate_view_state!(state, override {
///     set_bounds,
///     select,
///     unselect,
///     needs_redraw,
///     mark_redrawn,
/// });
///
/// fn set_bounds(&mut self, r: Rect) {
///     self.state.bounds = r;
///     self.state.dirty = true;
///     self.layout();
/// }
/// // ... other overridden methods ...
/// ```
///
/// Future additions to the macro propagate automatically — only explicitly
/// listed methods are excluded.
#[macro_export]
macro_rules! delegate_view_state {
    ($field:ident) => {
        fn bounds(&self) -> $crate::geometry::Rect {
            self.$field.bounds
        }
        fn set_bounds(&mut self, r: $crate::geometry::Rect) {
            self.$field.bounds = r;
            self.$field.dirty = true;
        }
        fn options(&self) -> $crate::view::ViewOptions {
            self.$field.options
        }
        fn title(&self) -> &str {
            &self.$field.title
        }
        fn needs_redraw(&self) -> bool {
            self.$field.dirty
        }
        fn mark_redrawn(&mut self) {
            self.$field.dirty = false;
        }
        fn select(&mut self) {
            self.$field.focused = true;
            self.$field.dirty = true;
        }
        fn unselect(&mut self) {
            self.$field.focused = false;
            self.$field.dirty = true;
        }
    };
    ($field:ident, override { $($skip:ident),* $(,)? }) => {
        $crate::__dvs_maybe!(bounds, [$($skip),*], {
            fn bounds(&self) -> $crate::geometry::Rect {
                self.$field.bounds
            }
        });
        $crate::__dvs_maybe!(set_bounds, [$($skip),*], {
            fn set_bounds(&mut self, r: $crate::geometry::Rect) {
                self.$field.bounds = r;
                self.$field.dirty = true;
            }
        });
        $crate::__dvs_maybe!(options, [$($skip),*], {
            fn options(&self) -> $crate::view::ViewOptions {
                self.$field.options
            }
        });
        $crate::__dvs_maybe!(title, [$($skip),*], {
            fn title(&self) -> &str {
                &self.$field.title
            }
        });
        $crate::__dvs_maybe!(needs_redraw, [$($skip),*], {
            fn needs_redraw(&self) -> bool {
                self.$field.dirty
            }
        });
        $crate::__dvs_maybe!(mark_redrawn, [$($skip),*], {
            fn mark_redrawn(&mut self) {
                self.$field.dirty = false;
            }
        });
        $crate::__dvs_maybe!(select, [$($skip),*], {
            fn select(&mut self) {
                self.$field.focused = true;
                self.$field.dirty = true;
            }
        });
        $crate::__dvs_maybe!(unselect, [$($skip),*], {
            fn unselect(&mut self) {
                self.$field.focused = false;
                self.$field.dirty = true;
            }
        });
    };
}

/// Internal: emit `$body` only if `$method` is NOT in the skip list.
/// Uses a tt-muncher that compares idents via dedicated match arms.
#[macro_export]
#[doc(hidden)]
macro_rules! __dvs_maybe {
    // Specific method names — if the method matches a skip entry, suppress.
    (bounds, [bounds $(, $rest:ident)*], { $($body:tt)* }) => {};
    (set_bounds, [set_bounds $(, $rest:ident)*], { $($body:tt)* }) => {};
    (options, [options $(, $rest:ident)*], { $($body:tt)* }) => {};
    (title, [title $(, $rest:ident)*], { $($body:tt)* }) => {};
    (needs_redraw, [needs_redraw $(, $rest:ident)*], { $($body:tt)* }) => {};
    (mark_redrawn, [mark_redrawn $(, $rest:ident)*], { $($body:tt)* }) => {};
    (select, [select $(, $rest:ident)*], { $($body:tt)* }) => {};
    (unselect, [unselect $(, $rest:ident)*], { $($body:tt)* }) => {};

    // Skip list head doesn't match — pop it and recurse.
    ($method:ident, [$head:ident $(, $rest:ident)*], { $($body:tt)* }) => {
        $crate::__dvs_maybe!($method, [$($rest),*], { $($body)* });
    };

    // Skip list exhausted — emit the body.
    ($method:ident, [], { $($body:tt)* }) => {
        $($body)*
    };
}
