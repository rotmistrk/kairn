//! StructuredView inline editing — start, commit, cancel via InputLine.

use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;

use super::{EditTarget, StructuredView};

impl StructuredView {
    /// Start inline editing for the current cursor position and column focus.
    pub(crate) fn start_edit(&mut self, target: EditTarget) {
        let Some(&node_id) = self.tree.data.visible_nodes.get(self.tree.cursor) else {
            return;
        };
        let text = match target {
            EditTarget::Value => self.tree.data.doc.value_display(node_id).to_owned(),
            EditTarget::Key => self.tree.data.doc.key(node_id).unwrap_or("").to_owned(),
            EditTarget::Meta => self.tree.data.doc.meta(node_id).to_owned(),
        };
        self.edit_target = target;
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(&text);
        input.select_all();
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        let mut boxed: Box<dyn View> = Box::new(input);
        boxed.set_sink(sink);
        boxed.set_palette(pal);
        boxed.select();
        self.input_line = Some(boxed);
        self.editing_row = Some(self.tree.cursor);
        self.tree.state.mark_dirty();
    }

    /// Commit the current inline edit.
    pub(crate) fn commit_edit(&mut self) -> Option<String> {
        let text = self.input_text();
        self.input_line = None;
        let row = self.editing_row.take()?;
        let &node_id = self.tree.data.visible_nodes.get(row)?;
        let result = match self.edit_target {
            EditTarget::Value => self.tree.data_mut().doc.set_value(node_id, &text),
            EditTarget::Key => self.tree.data_mut().doc.set_key(node_id, &text),
            EditTarget::Meta => {
                self.tree.data_mut().doc.set_meta(node_id, &text);
                Ok(())
            }
        };
        self.dirty = true;
        self.sync_title();
        self.rebuild_visible();
        self.tree.state.mark_dirty();
        result.err()
    }

    /// Cancel the current inline edit.
    pub(crate) fn cancel_edit(&mut self) {
        self.input_line = None;
        self.editing_row = None;
        self.tree.state.mark_dirty();
    }

    /// Whether editing is active.
    pub(crate) fn is_editing(&self) -> bool {
        self.editing_row.is_some()
    }

    /// Get text from the InputLine.
    pub(crate) fn input_text(&mut self) -> String {
        if let Some(input) = self.input_line.as_mut() {
            if let Some(il) = input.as_any_mut().and_then(|a| a.downcast_mut::<InputLine>()) {
                return il.text().to_string();
            }
        }
        String::new()
    }

    /// Palette for editing: Text = cursor row style.
    pub(crate) fn edit_palette(&self) -> Arc<dyn Palette> {
        let base = palette();
        let cursor_style = base.style(StyleId::CursorFocused);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, cursor_style))
    }

    /// Insert an InputLine with given text (used by sort-by-path and filter).
    pub(crate) fn start_input_line(&mut self, text: &str) {
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(text);
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        let mut boxed: Box<dyn View> = Box::new(input);
        boxed.set_sink(sink);
        boxed.set_palette(pal);
        boxed.select();
        self.input_line = Some(boxed);
        self.editing_row = Some(self.tree.cursor);
        self.tree.state.mark_dirty();
    }
}
