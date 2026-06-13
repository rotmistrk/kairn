//! Edit/filter mode logic for TodoTreeView.

use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;
use txv_widgets::tree_view::TreeData;
use txv_widgets::{CM_ACTIVATE_GROUP, CM_DEACTIVATE_GROUP};

use super::{TodoTreeView, TODO_STATUS_GROUP};

impl TodoTreeView {
    /// Start editing the current item title.
    pub(super) fn start_edit(&mut self) {
        let row = self.inner_mut().cursor();
        if row >= self.inner_mut().data_mut().visible_count() {
            return;
        }
        let id = self.inner_mut().data_mut().visible_id(row);
        let label = self.inner_mut().data_mut().label(id).to_owned();
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(&label);
        input.select_all();
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        let idx = self.group.child_count() - 1;
        self.group.set_focused_index(idx);
        if let Some(child) = self.group.child_mut(idx) {
            child.set_sink(sink);
            child.set_palette(pal);
            child.select();
        }
        self.editing_row = Some(row);
        self.layout_edit_child();
        self.group.mark_dirty();
        self.group
            .put_command(CM_DEACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    pub(super) fn start_edit_selected(&mut self) {
        self.start_edit();
    }

    /// Start filter mode.
    pub(super) fn start_filter(&mut self) {
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(self.inner_mut().data_mut().filter_text());
        let pal = self.filter_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        let idx = self.group.child_count() - 1;
        self.group.set_focused_index(idx);
        if let Some(child) = self.group.child_mut(idx) {
            child.set_sink(sink);
            child.set_palette(pal);
            child.select();
        }
        self.filter_active = true;
        let b = self.group.bounds();
        let draw_h = b.h().saturating_sub(1);
        self.group.set_child_bounds(0, Rect::new(0, 0, b.w(), draw_h));
        self.layout_edit_child();
        self.group.mark_dirty();
        self.group
            .put_command(CM_DEACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    /// Palette for item editing: Text = cursor row style (focused).
    fn edit_palette(&self) -> Arc<dyn Palette> {
        let base = palette();
        let cursor_style = base.style(StyleId::CursorFocused);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, cursor_style))
    }

    /// Palette for filter bar: Text = StatusBar style.
    fn filter_palette(&self) -> Arc<dyn Palette> {
        let base = palette();
        let sb_style = base.style(StyleId::StatusBar);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, sb_style))
    }

    /// Get the InputLine child mutably (child 1 when editing).
    pub(super) fn input_line_mut(&mut self) -> Option<&mut InputLine> {
        if self.group.child_count() > 1 {
            self.group
                .child_mut(1)
                .and_then(|c| c.as_any_mut()?.downcast_mut::<InputLine>())
        } else {
            None
        }
    }

    /// Remove the InputLine child (child 1).
    pub(super) fn remove_input_line(&mut self) {
        if self.group.child_count() > 1 {
            self.group.remove(1);
            self.group.set_focused_index(0);
        }
    }

    /// Commit the active edit.
    pub(super) fn commit_edit(&mut self) {
        let text = self.input_line_mut().map(|i| i.text().to_string()).unwrap_or_default();
        self.remove_input_line();
        if let Some(row) = self.editing_row.take() {
            self.inner_mut().data_mut().update_title(row, text);
        }
        self.group.mark_dirty();
        self.group
            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    /// Cancel the active edit.
    pub(super) fn cancel_edit(&mut self) {
        self.remove_input_line();
        self.editing_row = None;
        self.group.mark_dirty();
        self.group
            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    /// Commit filter (keep filter text, remove InputLine).
    pub(super) fn commit_filter(&mut self) {
        self.remove_input_line();
        self.filter_active = false;
        let b = self.group.bounds();
        self.group.set_child_bounds(0, Rect::new(0, 0, b.w(), b.h()));
        self.group.mark_dirty();
        self.group
            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    /// Cancel filter (clear filter text, remove InputLine).
    pub(super) fn cancel_filter(&mut self) {
        self.remove_input_line();
        self.filter_active = false;
        self.inner_mut().data_mut().clear_filter_text();
        self.inner_mut().data_mut().rebuild_flat();
        self.inner_mut().set_cursor(0);
        let b = self.group.bounds();
        self.group.set_child_bounds(0, Rect::new(0, 0, b.w(), b.h()));
        self.group.mark_dirty();
        self.group
            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    /// Whether we're in any editing mode.
    pub(super) fn is_editing(&self) -> bool {
        self.editing_row.is_some() || self.filter_active
    }

    /// Sync tree visibility — hide when empty, show when has items.
    pub(crate) fn sync_tree_visibility(&mut self) {
        let visible = self.inner_mut().data_mut().visible_count() > 0;
        self.group.set_child_visible(0, visible);
        self.group.mark_dirty();
    }

    /// Mark both group and tree child as needing redraw.
    pub(crate) fn mark_tree_dirty(&mut self) {
        self.inner_mut().state_mut().mark_dirty();
        self.group.mark_dirty();
    }

    /// Position the InputLine child (child 1) at the correct location.
    pub(super) fn layout_edit_child(&mut self) {
        if self.group.child_count() <= 1 {
            return;
        }
        let b = self.group.bounds();
        let w = b.w();
        let h = b.h();
        if self.filter_active {
            let filter_row = h.saturating_sub(1);
            self.group
                .set_child_bounds(1, Rect::new(1, filter_row, w.saturating_sub(1), 1));
        } else if let Some(row) = self.editing_row {
            let scroll_offset = self.inner_mut().scroll_offset();
            let has_filter = !self.inner_mut().data_mut().filter_text().is_empty();
            let draw_h = if has_filter {
                h.saturating_sub(1)
            } else {
                h
            };
            if row < scroll_offset || (row - scroll_offset) >= draw_h as usize {
                return;
            }
            let screen_y = (row - scroll_offset) as u16;
            let id = self.inner_mut().data_mut().visible_id(row);
            let depth = self.inner_mut().data_mut().depth(id);
            let indent = (depth * 2 + 2) as u16;
            self.group
                .set_child_bounds(1, Rect::new(indent, screen_y, w.saturating_sub(indent), 1));
        }
    }

    /// Commit any active edit on resize.
    pub(super) fn commit_edit_on_resize(&mut self) {
        if self.editing_row.is_some() {
            self.commit_edit();
        } else if self.filter_active {
            self.commit_filter();
        }
    }
}
