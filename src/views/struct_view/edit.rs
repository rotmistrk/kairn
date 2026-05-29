//! StructuredView inline editing — start, commit, cancel via InputLine child.

use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;

use super::{EditTarget, StructuredView};

impl StructuredView {
    /// Start inline editing for the current cursor position and column focus.
    pub(crate) fn start_edit(&mut self, target: EditTarget) {
        let Some(&node_id) = self.visible_nodes.get(self.cursor) else {
            return;
        };
        let text = match target {
            EditTarget::Value => self.doc.value_display(node_id).to_owned(),
            EditTarget::Key => self.doc.key(node_id).unwrap_or("").to_owned(),
            EditTarget::Meta => self.doc.meta(node_id).to_owned(),
        };
        self.edit_target = target;
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(&text);
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        if let Some(child) = self.group.child_mut(0) {
            child.set_sink(sink);
            child.set_palette(pal);
        }
        self.editing_row = Some(self.cursor);
        self.group.mark_dirty();
    }

    /// Commit the current inline edit.
    pub(crate) fn commit_edit(&mut self) -> Option<String> {
        let text = self.input_text();
        self.remove_input_line();
        let row = self.editing_row.take()?;
        let &node_id = self.visible_nodes.get(row)?;
        let result = match self.edit_target {
            EditTarget::Value => self.doc.set_value(node_id, &text),
            EditTarget::Key => self.doc.set_key(node_id, &text),
            EditTarget::Meta => {
                self.doc.set_meta(node_id, &text);
                Ok(())
            }
        };
        self.dirty = true;
        self.sync_title();
        self.group.mark_dirty();
        result.err()
    }

    /// Cancel the current inline edit.
    pub(crate) fn cancel_edit(&mut self) {
        self.remove_input_line();
        self.editing_row = None;
        self.group.mark_dirty();
    }

    /// Whether editing is active.
    pub(crate) fn is_editing(&self) -> bool {
        self.editing_row.is_some()
    }

    /// Get text from the InputLine child.
    pub(crate) fn input_text(&mut self) -> String {
        if self.group.child_count() > 0 {
            if let Some(child) = self.group.child_mut(0) {
                if let Some(input) = child.as_any_mut().and_then(|a| a.downcast_mut::<InputLine>()) {
                    return input.text().to_string();
                }
            }
        }
        String::new()
    }

    /// Remove the InputLine child.
    pub(crate) fn remove_input_line(&mut self) {
        if self.group.child_count() > 0 {
            self.group.remove(0);
        }
    }

    /// Palette for editing: Text = cursor row style.
    pub(crate) fn edit_palette(&self) -> Arc<dyn Palette> {
        let base = palette();
        let cursor_style = base.style(StyleId::CursorFocused);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, cursor_style))
    }

    /// Insert an InputLine child with given text (used by sort-by-path and filter).
    pub(crate) fn start_input_line(&mut self, text: &str) {
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(text);
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        if let Some(child) = self.group.child_mut(0) {
            child.set_sink(sink);
            child.set_palette(pal);
        }
        self.editing_row = Some(self.cursor);
        self.group.mark_dirty();
    }
}
