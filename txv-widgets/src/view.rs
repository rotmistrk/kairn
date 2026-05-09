//! View hierarchy — core trait and types.
//!
//! A View is a rectangular UI element that can draw itself and handle events.
//! Views form a tree: a Group is a View that contains child Views.

use crossterm::event::KeyEvent;
use txv::layout::Rect;
use txv::surface::Surface;

/// A command identifier. Views communicate via commands, not by
/// knowing about each other.
pub type CommandId = u16;

/// Well-known commands.
pub mod commands {
    use super::CommandId;

    pub const CM_QUIT: CommandId = 1;
    pub const CM_OPEN_FILE: CommandId = 2;
    pub const CM_SAVE: CommandId = 3;
    pub const CM_CLOSE: CommandId = 4;
    pub const CM_FOCUS_NEXT: CommandId = 5;
    pub const CM_FOCUS_PREV: CommandId = 6;
    pub const CM_RESIZE_LEFT: CommandId = 10;
    pub const CM_RESIZE_RIGHT: CommandId = 11;
    pub const CM_RESIZE_UP: CommandId = 12;
    pub const CM_RESIZE_DOWN: CommandId = 13;
}

/// An event that flows through the view tree.
#[derive(Debug)]
pub enum Event {
    /// Keyboard input.
    Key(KeyEvent),
    /// Terminal resized.
    Resize(u16, u16),
    /// Command from menu, keybinding, or another view.
    Command(CommandId),
    /// Timer tick.
    Tick,
    /// Data available from an async source (PTY, LSP, etc.).
    Data { source_id: usize, payload: Vec<u8> },
}

/// Result of handling an event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandleResult {
    /// Event was consumed — stop dispatching.
    Consumed,
    /// Event was not handled — continue dispatching.
    Ignored,
}

/// Grow flags: how a view resizes when its parent resizes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrowFlags(u8);

impl GrowFlags {
    pub const NONE: Self = Self(0);
    pub const GROW_X: Self = Self(1);
    pub const GROW_Y: Self = Self(2);
    pub const SLIDE_X: Self = Self(4);
    pub const SLIDE_Y: Self = Self(8);

    /// Combine two flag sets.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Check if a flag is set.
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

/// Shared context passed during drawing.
pub struct DrawContext {
    /// Whether the application window is focused.
    pub app_focused: bool,
    /// Tick counter (for animations like cursor blink).
    pub tick: u64,
}

/// A rectangular UI element that can draw and handle events.
pub trait View: Send {
    /// Draw this view into the given surface.
    /// The surface is exactly this view's bounds — no need to offset.
    fn draw(&self, surface: &mut Surface<'_>, ctx: &DrawContext);

    /// Handle an event. Return whether it was consumed.
    fn handle(&mut self, event: &Event) -> HandleResult;

    /// This view's bounds (position + size) within its parent.
    fn bounds(&self) -> Rect;

    /// Change this view's bounds (called by parent during layout).
    fn set_bounds(&mut self, rect: Rect);

    /// Whether this view can receive focus.
    fn focusable(&self) -> bool {
        true
    }

    /// Grow flags: how this view resizes when parent resizes.
    fn grow_flags(&self) -> GrowFlags {
        GrowFlags::NONE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_result_eq() {
        assert_eq!(HandleResult::Consumed, HandleResult::Consumed);
        assert_ne!(HandleResult::Consumed, HandleResult::Ignored);
    }

    #[test]
    fn grow_flags_none() {
        assert_eq!(GrowFlags::NONE.0, 0);
    }

    #[test]
    fn grow_flags_union() {
        let f = GrowFlags::GROW_X.union(GrowFlags::GROW_Y);
        assert!(f.contains(GrowFlags::GROW_X));
        assert!(f.contains(GrowFlags::GROW_Y));
        assert!(!f.contains(GrowFlags::SLIDE_X));
    }

    #[test]
    fn event_variants() {
        let _k = Event::Key(KeyEvent::new(
            crossterm::event::KeyCode::Char('a'),
            crossterm::event::KeyModifiers::NONE,
        ));
        let _r = Event::Resize(80, 24);
        let _c = Event::Command(commands::CM_QUIT);
        let _t = Event::Tick;
        let _d = Event::Data {
            source_id: 0,
            payload: vec![1, 2, 3],
        };
    }

    #[test]
    fn command_ids_distinct() {
        assert_ne!(commands::CM_QUIT, commands::CM_OPEN_FILE);
        assert_ne!(commands::CM_FOCUS_NEXT, commands::CM_FOCUS_PREV);
    }
}
