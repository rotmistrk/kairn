//! TodoTreeView key dispatch — normal mode.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;
use txv_widgets::CM_ACTIVATE_GROUP;

use super::handle::{self, HandleAction};
use super::{TodoTreeView, TODO_STATUS_GROUP};
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};

impl TodoTreeView {
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
                    self.group
                        .put_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ConfirmContext::TodoDelete)));
                    let msg = "Delete item? [y]es [Esc]cancel".to_string();
                    self.group.put_command(CM_CONFIRM, Some(Box::new(msg)));
                }
                self.apply_action(action);
                self.emit_note_update_if_cursor_changed(prev_cursor);
                return HandleResult::Consumed;
            }
        }
        let result = self.inner.handle(event);
        if result == HandleResult::Consumed {
            self.group.mark_dirty();
        }
        self.emit_note_update_if_cursor_changed(prev_cursor);
        result
    }

    pub(super) fn drain_edit_commands(&mut self) {
        for ev in self.child_sink.drain() {
            if let Event::Command { id, data, .. } = ev {
                match id {
                    CM_OK => {
                        let text = data
                            .and_then(|d| d.downcast::<String>().ok())
                            .map(|s| *s)
                            .unwrap_or_default();
                        let row = self.editing_row.take().unwrap_or(0);
                        self.remove_input_line();
                        self.inner.data.update_title(row, text);
                        self.group.mark_dirty();
                        self.group
                            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
                        return;
                    }
                    CM_CANCEL => {
                        self.cancel_edit();
                        return;
                    }
                    _ => {}
                }
            }
        }
    }

    pub(super) fn drain_child_sink(&mut self) {
        self.child_sink.drain();
    }

    pub(super) fn handle_filter_key(&mut self, key: &KeyEvent, event: &Event) -> HandleResult {
        let result = self.group.dispatch(event);
        self.drain_child_sink();
        if let Some(input) = self.input_line_mut() {
            self.inner.data.filter_text = input.text().to_string();
        }
        self.inner.data.rebuild_flat();
        self.inner.set_cursor(0);
        self.group.mark_dirty();
        if key.code == KeyCode::Esc {
            self.cancel_filter();
        } else if key.code == KeyCode::Enter {
            self.commit_filter();
        }
        if result == HandleResult::Consumed {
            result
        } else {
            HandleResult::Consumed
        }
    }
}
