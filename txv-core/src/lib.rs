//! # txv-core
//!
//! Pure Rust TUI framework core. Zero external dependencies.
//! Defines the View trait, Group three-phase dispatch, Surface, EventQueue,
//! Backend trait, and the run loop.
//!
//! ## How to create a View
//!
//! ```rust
//! use txv_core::prelude::*;
//!
//! struct MyView {
//!     state: ViewState,
//! }
//!
//! impl View for MyView {
//!     delegate_view_state!(state);
//!
//!     fn draw(&self, surface: &mut Surface) {
//!         surface.print(0, 0, "Hello", Style::default());
//!     }
//!
//!     fn handle(
//!         &mut self,
//!         event: &Event,
//!         queue: &mut EventQueue,
//!     ) -> HandleResult {
//!         HandleResult::Ignored
//!     }
//! }
//! ```
//!
//! ## Overriding delegated methods
//!
//! When a widget needs custom behavior for some View methods, list them
//! in an `override` block. The macro generates everything except those:
//!
//! ```rust,ignore
//! impl View for MyContainer {
//!     delegate_view_state!(state, override { set_bounds, needs_redraw });
//!
//!     fn set_bounds(&mut self, r: Rect) {
//!         self.state.bounds = r;
//!         self.state.dirty = true;
//!         self.relayout_children();
//!     }
//!
//!     fn needs_redraw(&self) -> bool {
//!         self.state.dirty || self.child.needs_redraw()
//!     }
//!
//!     // draw() and handle() as usual...
//! }
//! ```

pub mod cell;
pub mod commands;
pub mod complete;
pub mod dialog;
pub mod event;
pub mod geometry;
pub mod group;
pub mod program;
pub mod run;
pub mod surface;
pub mod view;
pub mod window;

/// Prelude — import everything needed to implement a View.
pub mod prelude {
    pub use crate::cell::{Attrs, Cell, Color, Style};
    pub use crate::commands::*;
    pub use crate::complete::{Completer, Completion};
    pub use crate::dialog::DialogState;
    pub use crate::event::{
        CommandId, Event, KeyCode, KeyEvent, KeyMod, MouseAction,
        MouseButton, MouseEvent,
    };
    pub use crate::geometry::{Point, Rect};
    pub use crate::group::GroupState;
    pub use crate::run::{exec_view, run, run_cycles, Backend, MockBackend};
    pub use crate::surface::{SubSurface, Surface};
    pub use crate::view::{
        EventQueue, HandleResult, View, ViewOptions, ViewState,
    };
    pub use crate::window::{FrameStyle, WindowState};

    // Re-export macros (they are already at crate root via #[macro_export])
    pub use crate::{
        delegate_dialog_state, delegate_group_state,
        delegate_view_state, delegate_window_state,
    };
}
