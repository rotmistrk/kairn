//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::inline_edit::{InlineEditResult, InlineEditor};
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use self::handle::HandleAction;

mod apply_action;
pub mod data;
mod draw;
mod flat_node;
mod handle;
mod mcp;
pub mod model;

pub use self::data::TodoTreeData;

/// The todo tree view — wraps TreeView<TodoTreeData>.
pub struct TodoTreeView {
    inner: TreeView<TodoTreeData>,
    editing: Option<InlineEditor>,
    filter_editor: Option<InlineEditor>,
    /// Pending crypto path for passphrase prompt.
    crypto_pending: Option<CryptoPending>,
}

enum CryptoPending {
    Encrypt(model::TreePath),
    Decrypt(model::TreePath),
}

impl TodoTreeView {
    pub fn new(root: &Path) -> Self {
        let todo_path = root.join(".kairn.todo");
        let data = TodoTreeData::new(&todo_path);
        Self {
            inner: TreeView::new(data),
            editing: None,
            filter_editor: None,
            crypto_pending: None,
        }
    }

    /// Emit CM_TODO_NOTE_UPDATE if cursor moved to a different item.
    fn emit_note_update_if_cursor_changed(&mut self, prev_cursor: usize) {
        if self.inner.cursor == prev_cursor {
            return;
        }
        let cursor = self.inner.cursor;
        if cursor >= self.inner.data.visible_count() {
            return;
        }
        let id = self.inner.data.visible_id(cursor);
        if let Some(path) = self.inner.data.path_at(id) {
            let path = path.clone();
            if let Some(item) = model::get_item(&self.inner.data.file, &path) {
                let note = item.note.clone();
                self.inner
                    .state
                    .put_command(crate::commands::CM_TODO_NOTE_UPDATE, Some(Box::new((path, note))));
            }
        }
    }

    /// Execute the pending delete (called from handler_confirm on 'y' response).
    pub fn confirm_delete_execute(&mut self) {
        let cursor = self.inner.cursor;
        if cursor < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(cursor);
            if let Some(path) = self.inner.data.path_at(id) {
                let path = path.clone();
                model::remove_item(&mut self.inner.data.file, &path);
                model::propagate_completion(&mut self.inner.data.file, &path);
                self.inner.data.save();
                self.inner.data.rebuild_flat();
                self.inner.mark_dirty();
            }
        }
    }

    /// Execute crypto passphrase response (called from handler_confirm on 'y'/commit).
    pub fn crypto_passphrase_response(&mut self, passphrase: &str) {
        let Some(pending) = self.crypto_pending.take() else {
            return;
        };
        match pending {
            CryptoPending::Encrypt(path) => {
                if let Some(item) = model::get_item_mut(&mut self.inner.data.file, &path) {
                    if let Err(e) = duir_core::crypto::encrypt_item(item, passphrase) {
                        log::warn!("encrypt failed: {e}");
                    }
                }
            }
            CryptoPending::Decrypt(path) => {
                if let Some(item) = model::get_item_mut(&mut self.inner.data.file, &path) {
                    if let Err(e) = duir_core::crypto::decrypt_item(item, passphrase) {
                        log::warn!("decrypt failed: {e}");
                    }
                }
            }
        }
        self.inner.data.save();
        self.inner.data.rebuild_flat();
        self.inner.mark_dirty();
    }

    fn start_edit(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new_selected(row, &label));
            self.inner.mark_dirty();
        }
    }

    fn start_edit_selected(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new_selected(row, &label));
            self.inner.mark_dirty();
        }
    }

    fn handle_editing_key(&mut self, key: &KeyEvent) -> HandleResult {
        let Some(ref mut editor) = self.editing else {
            return HandleResult::Ignored;
        };
        match editor.handle_key(key) {
            InlineEditResult::Continue => {}
            InlineEditResult::Commit(text) => {
                let row = editor.row;
                self.editing = None;
                self.inner.data.update_title(row, text);
            }
            InlineEditResult::Cancel => {
                self.editing = None;
            }
        }
        self.inner.mark_dirty();
        HandleResult::Consumed
    }
}

impl View for TodoTreeView {
    delegate_view!(inner, override { title, handle, draw, can_close });

    fn title(&self) -> &str {
        "Todo"
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn draw(&mut self) {
        self.draw_tree();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            if self.inner.data.reload_if_changed() {
                self.inner.mark_dirty();
            }
            return self.inner.handle(event);
        }
        let Event::Key(key) = event else {
            return self.inner.handle(event);
        };
        // Filter editor takes priority
        if self.filter_editor.is_some() {
            // Special: Esc exits filter
            if key.code == KeyCode::Esc {
                self.inner.data.filter_text.clear();
                self.inner.data.rebuild_flat();
                self.inner.cursor = 0;
                self.filter_editor = None;
                self.inner.mark_dirty();
                return HandleResult::Consumed;
            }
            // Enter commits filter (keeps text, exits editor)
            if key.code == KeyCode::Enter {
                self.filter_editor = None;
                self.inner.mark_dirty();
                return HandleResult::Consumed;
            }
            // Pass to filter editor
            if let Some(ref mut editor) = self.filter_editor {
                editor.handle_key(key);
                self.inner.data.filter_text = editor.buffer.clone();
                self.inner.data.rebuild_flat();
                self.inner.cursor = 0;
                self.inner.mark_dirty();
            }
            return HandleResult::Consumed;
        }
        if self.editing.is_some() {
            return self.handle_editing_key(key);
        }
        // 'n' works even on empty tree — adds first item
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
                    self.inner.state.put_command(
                        crate::commands::CM_SET_CONFIRM_CONTEXT,
                        Some(Box::new(crate::commands::ConfirmContext::TodoDelete)),
                    );
                    self.inner.state.put_command(
                        crate::commands::CM_CONFIRM,
                        Some(Box::new("Delete item? [y]es [Esc]cancel".to_string())),
                    );
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
