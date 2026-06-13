//! StructuredView inline editing — start, commit, cancel via InputLine.

use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;

use super::{EditTarget, StructuredView};

impl StructuredView {
    /// Start inline editing for the current cursor position and column focus.
    pub(crate) fn start_edit(&mut self, target: EditTarget) {
        let cursor = self.inner().cursor();
        let Some(&node_id) = self.inner_mut().data_mut().visible_nodes().get(cursor) else {
            return;
        };
        let text = match target {
            EditTarget::Value => self.inner_mut().data_mut().doc().value_display(node_id).to_owned(),
            EditTarget::Key => self.inner_mut().data_mut().doc().key(node_id).unwrap_or("").to_owned(),
            EditTarget::Meta => self.inner_mut().data_mut().doc().meta(node_id).to_owned(),
        };
        self.edit_target = target;
        self.insert_input_child(&text);
        self.editing_row = Some(self.inner().cursor());
        self.layout_input_line();
        self.group.mark_dirty();
    }

    /// Commit the current inline edit.
    pub(crate) fn commit_edit(&mut self) -> Option<String> {
        let text = self.input_text();
        self.remove_input_child();
        let row = self.editing_row.take()?;
        let &node_id = self.inner_mut().data_mut().visible_nodes().get(row)?;
        let result = match self.edit_target {
            EditTarget::Value => self.inner_mut().data_mut().doc_mut().set_value(node_id, &text),
            EditTarget::Key => self.inner_mut().data_mut().doc_mut().set_key(node_id, &text),
            EditTarget::Meta => {
                self.inner_mut().data_mut().doc_mut().set_meta(node_id, &text);
                Ok(())
            }
        };
        self.dirty = true;
        self.sync_title();
        self.rebuild_visible();
        self.group.mark_dirty();
        result.err()
    }

    /// Cancel the current inline edit.
    pub(crate) fn cancel_edit(&mut self) {
        self.remove_input_child();
        self.editing_row = None;
        self.group.mark_dirty();
    }

    /// Whether editing is active.
    pub(crate) fn is_editing(&self) -> bool {
        self.editing_row.is_some()
    }

    /// Get text from the InputLine child (child 1).
    pub(crate) fn input_text(&mut self) -> String {
        self.group
            .child_mut(1)
            .and_then(|c| c.as_any_mut())
            .and_then(|a| a.downcast_mut::<InputLine>())
            .map(|il| il.text().to_string())
            .unwrap_or_default()
    }

    /// Palette for editing: Text = cursor row style.
    pub(crate) fn edit_palette(&self) -> Arc<dyn Palette> {
        let base = palette();
        let cursor_style = base.style(StyleId::CursorFocused);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, cursor_style))
    }

    /// Insert an InputLine with given text (used by sort-by-path and filter).
    pub(crate) fn start_input_line(&mut self, text: &str) {
        self.insert_input_child(text);
        self.editing_row = Some(self.inner().cursor());
        self.layout_input_line();
        self.group.mark_dirty();
    }

    /// Insert InputLine as child 1 of the group (tree is always child 0).
    fn insert_input_child(&mut self, text: &str) {
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(text);
        input.select_all();
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        self.group.set_focused_index(1);
        if let Some(child) = self.group.child_mut(1) {
            child.set_sink(sink);
            child.set_palette(pal);
            child.select();
        }
    }

    /// Remove InputLine child from the group (index 1).
    pub(crate) fn remove_input_child(&mut self) {
        if self.group.child_count() > 1 {
            self.group.remove(1);
        }
    }
}
