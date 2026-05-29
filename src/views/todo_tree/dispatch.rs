//! TodoTreeView key dispatch — filter and normal mode.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

use super::handle::{self, HandleAction};
use super::TodoTreeView;
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};

impl TodoTreeView {
    pub(super) fn handle_filter_key(&mut self, key: &KeyEvent) -> HandleResult {
        if key.code == KeyCode::Esc {
            self.inner.data.filter_text.clear();
            self.inner.data.rebuild_flat();
            self.inner.cursor = 0;
            self.filter_editor = None;
            self.inner.mark_dirty();
            return HandleResult::Consumed;
        }
        if key.code == KeyCode::Enter {
            self.filter_editor = None;
            self.inner.mark_dirty();
            return HandleResult::Consumed;
        }
        if let Some(ref mut input) = self.filter_editor {
            input.handle(&Event::Key(*key));
            self.edit_sink.drain();
            self.inner.data.filter_text = input.text().to_string();
            self.inner.data.rebuild_flat();
            self.inner.cursor = 0;
            self.inner.mark_dirty();
        }
        HandleResult::Consumed
    }

    pub(super) fn handle_normal_key(&mut self, key: &KeyEvent, event: &Event) -> HandleResult {
        if key.code == KeyCode::Char('n') && self.inner.data.visible_count() == 0 {
            self.inner.data.add_first_item();
            return HandleResult::Consumed;
        }
        if key.code == KeyCode::Char('e') && self.inner.data.visible_count() > 0 {
            self.start_edit();
            return HandleResult::Consumed;
        }
        let prev_cursor = self.inner.cursor;
        let cursor = self.inner.cursor;
        if self.inner.data.visible_count() > 0 {
            if let Some(action) = handle::handle_todo_key(key, &mut self.inner.data, cursor) {
                if matches!(action, HandleAction::ConfirmDelete) {
                    self.inner
                        .state
                        .put_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ConfirmContext::TodoDelete)));
                    let msg = "Delete item? [y]es [Esc]cancel".to_string();
                    self.inner.state.put_command(CM_CONFIRM, Some(Box::new(msg)));
                }
                self.apply_action(action);
                self.emit_note_update_if_cursor_changed(prev_cursor);
                return HandleResult::Consumed;
            }
        }
        let result = self.inner.handle(event);
        self.emit_note_update_if_cursor_changed(prev_cursor);
        result
    }
}
