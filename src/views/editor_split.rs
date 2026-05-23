//! EditorSplit — wraps SplitPanel for editor-specific split behavior.
//!
//! Intercepts Ctrl-W w to switch focus between panes.
//! The handler creates this when :split/:vsplit is invoked.

use txv_core::prelude::*;
use txv_widgets::split_panel::SplitPanel;
use txv_widgets::tiled_workspace::types::SplitDir;

use crate::views::editor::EditorView;
use crate::views::scroll_map::ScrollMap;

/// A split editor view — two EditorViews side by side or stacked.
pub struct EditorSplit {
    split: SplitPanel,
    /// When true, scrolling in one pane scrolls the other.
    linked_scroll: bool,
    /// Hunk-aligned scroll map: maps line in pane 0 → line in pane 1.
    scroll_map: Option<ScrollMap>,
}

impl EditorSplit {
    pub fn new(direction: SplitDir, first: Box<dyn View>, second: Box<dyn View>) -> Self {
        let mut split = SplitPanel::new(direction);
        split.add_child(first, 0.5);
        split.add_child(second, 0.5);
        Self {
            split,
            linked_scroll: false,
            scroll_map: None,
        }
    }

    /// Change orientation without recreating the split.
    pub fn set_direction(&mut self, direction: SplitDir) {
        self.split.set_direction(direction);
    }

    /// Get the split direction.
    pub fn direction(&self) -> SplitDir {
        self.split.direction()
    }

    /// Access a child by index.
    pub fn child_mut(&mut self, idx: usize) -> Option<&mut Box<dyn View>> {
        self.split.child_mut(idx)
    }

    /// Set the focused pane index.
    pub fn set_focused(&mut self, idx: usize) {
        self.split.set_focused(idx);
    }

    /// Cycle focus between panes.
    pub fn cycle_focus(&mut self) {
        self.split.cycle_focus();
    }

    /// Enable linked scrolling with an optional scroll map.
    pub fn set_linked_scroll(&mut self, enabled: bool, map: Option<ScrollMap>) {
        self.linked_scroll = enabled;
        self.scroll_map = map;
    }

    /// Whether linked scrolling is enabled.
    pub fn is_linked_scroll(&self) -> bool {
        self.linked_scroll
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
    pub fn collapse(mut self) -> Option<Box<dyn View>> {
        let keep = 1 - self.split.focused_index();
        self.split.remove_child(keep).or_else(|| self.split.remove_child(0))
    }

    /// Remove the focused child, returning it.
    pub fn take_focused(&mut self) -> Option<Box<dyn View>> {
        let idx = self.split.focused_index();
        self.split.remove_child(idx)
    }

    /// Sync scroll position from focused pane to the other pane.
    fn sync_scroll(&mut self) {
        if self.split.child_count() < 2 {
            return;
        }
        let focused = self.split.focused_index();
        let other = 1 - focused;
        let scroll = self
            .split
            .child_mut(focused)
            .and_then(|v| v.as_any_mut())
            .and_then(|a| a.downcast_ref::<EditorView>())
            .map(|ev| ev.editor.viewport_scroll);
        let Some(scroll_top) = scroll else {
            return;
        };
        let target = match &self.scroll_map {
            Some(map) => map.translate(focused, scroll_top),
            None => scroll_top,
        };
        if let Some(other_view) = self.split.child_mut(other) {
            if let Some(ev) = other_view.as_any_mut().and_then(|a| a.downcast_mut::<EditorView>()) {
                ev.editor.viewport_scroll = target;
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
                self.split.cycle_focus();
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
