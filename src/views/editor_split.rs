//! EditorSplit — wraps SplitPane for editor-specific split behavior.
//!
//! Intercepts Ctrl-W w to switch focus between panes.
//! The handler creates this when :split/:vsplit is invoked.

use txv_core::prelude::*;
use txv_widgets::split_pane::{SplitDirection, SplitPane};

use crate::views::editor::EditorView;

/// A split editor view — two EditorViews side by side or stacked.
pub struct EditorSplit {
    pub split: SplitPane,
    /// When true, scrolling in one pane scrolls the other.
    pub linked_scroll: bool,
}

impl EditorSplit {
    pub fn new(direction: SplitDirection, first: Box<dyn View>, second: Box<dyn View>) -> Self {
        Self {
            split: SplitPane::new(direction, first, second),
            linked_scroll: false,
        }
    }

    /// Change orientation without recreating the split.
    pub fn set_direction(&mut self, direction: SplitDirection) {
        self.split.direction = direction;
        // Re-apply layout with new direction
        let bounds = self.split.bounds();
        self.split.set_bounds(bounds);
    }

    /// Get the focused pane as EditorView (if it is one).
    pub fn focused_editor_mut(&mut self) -> Option<&mut EditorView> {
        let idx = self.split.focused_index();
        self.split.child_mut(idx)?.as_any_mut()?.downcast_mut::<EditorView>()
    }

    /// Get the focused child index.
    pub fn focused_index(&self) -> usize {
        self.split.focused_index()
    }

    /// Remove the focused pane and return the other one.
    pub fn collapse(self) -> Box<dyn View> {
        let keep = if self.split.focused_index() == 0 {
            0
        } else {
            1
        };
        self.split.take_child(keep)
    }

    /// Remove the focused child, returning it. Leaves the split in an invalid state.
    /// Caller must replace the split with the returned view.
    pub fn take_focused(&mut self) -> Option<Box<dyn View>> {
        let idx = self.split.focused_index();
        Some(self.split.remove_child(idx))
    }

    /// Sync scroll position from focused pane to the other pane.
    fn sync_scroll(&mut self) {
        let focused = self.split.focused_index();
        let other = 1 - focused;
        let scroll = self
            .split
            .child_mut(focused)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_ref::<EditorView>())
            .map(|ev| ev.editor.viewport_scroll);
        if let Some(scroll_top) = scroll {
            if let Some(other_view) = self.split.child_mut(other) {
                if let Some(ev) = other_view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                    ev.editor.viewport_scroll = scroll_top;
                }
            }
        }
    }
}

impl View for EditorSplit {
    fn bounds(&self) -> Rect {
        self.split.bounds()
    }

    fn set_bounds(&mut self, r: Rect) {
        self.split.set_bounds(r);
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.split.set_sink(sink);
    }

    fn options(&self) -> ViewOptions {
        self.split.options()
    }

    fn select(&mut self) {
        self.split.select();
    }

    fn unselect(&mut self) {
        self.split.unselect();
    }

    fn title(&self) -> &str {
        ""
    }

    fn needs_redraw(&self) -> bool {
        self.split.needs_redraw()
    }

    fn draw(&mut self) {
        self.split.draw();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Intercept Ctrl-W w to switch focus between panes
        if let Event::Key(ke) = event {
            if ke.code == KeyCode::Char('w')
                && ke.modifiers
                    == (KeyMod {
                        ctrl: true,
                        alt: false,
                        shift: false,
                    })
            {
                self.split.focus_next();
                return HandleResult::Consumed;
            }
        }
        let result = self.split.handle(event);
        if self.linked_scroll {
            self.sync_scroll();
        }
        result
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn buffer(&self) -> &Buffer {
        self.split.buffer()
    }
}
