//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::inline_edit::{InlineEditResult, InlineEditor};
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use self::handle::HandleAction;

pub mod data;
mod draw;
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
                self.inner.state.mark_dirty();
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
        self.inner.state.mark_dirty();
    }

    fn start_edit(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new(row, &label));
            self.inner.state.mark_dirty();
        }
    }

    fn start_edit_selected(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new_selected(row, &label));
            self.inner.state.mark_dirty();
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
        self.inner.state.mark_dirty();
        HandleResult::Consumed
    }

    fn apply_action(&mut self, action: HandleAction, queue: &mut EventQueue) {
        match action {
            HandleAction::Stay => {}
            HandleAction::MoveDown => {
                let max = self.inner.data.visible_count().saturating_sub(1);
                if self.inner.cursor < max {
                    self.inner.cursor += 1;
                }
            }
            HandleAction::MoveTo(row) => {
                self.inner.cursor = row;
            }
            HandleAction::EditNew(row) => {
                self.inner.cursor = row;
                self.start_edit_selected();
            }
            HandleAction::ConfirmDelete => {
                // Handled below via CM_CONFIRM
            }
            HandleAction::EnterFilter => {
                self.filter_editor = Some(InlineEditor::new(0, &self.inner.data.filter_text));
            }
            HandleAction::CryptoEncrypt(path) => {
                self.crypto_pending = Some(CryptoPending::Encrypt(path));
                queue.put_command(
                    crate::commands::CM_SET_CONFIRM_CONTEXT,
                    Some(Box::new(crate::commands::ConfirmContext::TodoCrypto)),
                );
                queue.put_command(crate::commands::CM_CONFIRM, Some(Box::new("Passphrase: ".to_string())));
            }
            HandleAction::CryptoDecrypt(path) => {
                self.crypto_pending = Some(CryptoPending::Decrypt(path));
                queue.put_command(
                    crate::commands::CM_SET_CONFIRM_CONTEXT,
                    Some(Box::new(crate::commands::ConfirmContext::TodoCrypto)),
                );
                queue.put_command(crate::commands::CM_CONFIRM, Some(Box::new("Passphrase: ".to_string())));
            }
            HandleAction::OpenNote(path, note) => {
                queue.put_command(crate::commands::CM_TODO_NOTE_OPEN, Some(Box::new((path, note))));
            }
        }
        self.inner.state.mark_dirty();
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

    fn draw(&self, surface: &mut Surface) {
        self.draw_tree(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return self.inner.handle(event, queue);
        };
        // Filter editor takes priority
        if self.filter_editor.is_some() {
            // Special: Esc exits filter
            if key.code == KeyCode::Esc {
                self.inner.data.filter_text.clear();
                self.inner.data.rebuild_flat();
                self.inner.cursor = 0;
                self.filter_editor = None;
                self.inner.state.mark_dirty();
                return HandleResult::Consumed;
            }
            // Enter commits filter (keeps text, exits editor)
            if key.code == KeyCode::Enter {
                self.filter_editor = None;
                self.inner.state.mark_dirty();
                return HandleResult::Consumed;
            }
            // Pass to filter editor
            if let Some(ref mut editor) = self.filter_editor {
                editor.handle_key(key);
                self.inner.data.filter_text = editor.buffer.clone();
                self.inner.data.rebuild_flat();
                self.inner.cursor = 0;
                self.inner.state.mark_dirty();
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
        let cursor = self.inner.cursor;
        if self.inner.data.visible_count() > 0 {
            if let Some(action) = handle::handle_todo_key(key, &mut self.inner.data, cursor, queue) {
                if matches!(action, HandleAction::ConfirmDelete) {
                    queue.put_command(
                        crate::commands::CM_SET_CONFIRM_CONTEXT,
                        Some(Box::new(crate::commands::ConfirmContext::TodoDelete)),
                    );
                    queue.put_command(
                        crate::commands::CM_CONFIRM,
                        Some(Box::new("Delete item? [y]es [Esc]cancel".to_string())),
                    );
                }
                self.apply_action(action, queue);
                return HandleResult::Consumed;
            }
        }
        self.inner.handle(event, queue)
    }
}
