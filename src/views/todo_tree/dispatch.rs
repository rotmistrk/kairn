//! TodoTreeView key dispatch — normal mode.

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;
use txv_widgets::CM_ACTIVATE_GROUP;

use super::handle::{self, HandleAction};
use super::{TodoTreeView, TODO_STATUS_GROUP};
use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT};

impl TodoTreeView {
    pub(super) fn handle_normal_key(&mut self, key: &KeyEvent, event: &Event) -> HandleResult {
        if key.code() == KeyCode::Esc && !self.inner_mut().data_mut().filter_text().is_empty() {
            self.inner_mut().data_mut().clear_filter_text();
            self.inner_mut().data_mut().rebuild_flat();
            self.inner_mut().set_cursor(0);
            self.mark_tree_dirty();
            return HandleResult::Consumed;
        }
        if key.code() == KeyCode::Char('n') && self.inner_mut().data_mut().visible_count() == 0 {
            self.inner_mut().data_mut().add_first_item();
            return HandleResult::Consumed;
        }
        if key.code() == KeyCode::Char('e') && self.inner_mut().data_mut().visible_count() > 0 {
            self.start_edit();
            return HandleResult::Consumed;
        }
        let prev_cursor = self.inner_mut().cursor();
        let cursor = self.inner_mut().cursor();
        if self.inner_mut().data_mut().visible_count() > 0 {
            if let Some(action) = handle::handle_todo_key(key, self.inner_mut().data_mut(), cursor) {
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
        let result = self.inner_mut().handle(event);
        if result == HandleResult::Consumed {
            self.mark_tree_dirty();
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
                        self.inner_mut().data_mut().update_title(row, text);
                        self.mark_tree_dirty();
                        self.group
                            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
                        return;
                    }
                    CM_CANCEL => {
                        self.cancel_edit();
                        return;
                    }
                    _ => {
                        self.group.put_command(id, data);
                    }
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
        let text = self.input_line_mut().map(|input| input.text().to_string());
        if let Some(text) = text {
            self.inner_mut().data_mut().set_filter_text(text);
        }
        self.inner_mut().data_mut().rebuild_flat();
        self.inner_mut().set_cursor(0);
        self.mark_tree_dirty();
        if key.code() == KeyCode::Esc {
            self.cancel_filter();
        } else if key.code() == KeyCode::Enter {
            self.commit_filter();
        }
        if result == HandleResult::Consumed {
            result
        } else {
            HandleResult::Consumed
        }
    }
}
