//! TodoTreeView operations — confirm delete, crypto, note updates.

use duir_core::crypto::{decrypt_item, encrypt_item};
use txv_widgets::tree_view::TreeData;

use super::{model, CryptoPending, TodoTreeView};
use crate::commands::CM_TODO_NOTE_UPDATE;

impl TodoTreeView {
    pub(super) fn emit_note_update_if_cursor_changed(&mut self, prev_cursor: usize) {
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
                self.group
                    .put_command(CM_TODO_NOTE_UPDATE, Some(Box::new((path, note))));
            }
        }
    }

    /// Execute the pending delete.
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
                self.group.mark_dirty();
            }
        }
    }

    /// Execute crypto passphrase response.
    pub fn crypto_passphrase_response(&mut self, passphrase: &str) {
        let Some(pending) = self.crypto_pending.take() else {
            return;
        };
        match pending {
            CryptoPending::Encrypt(path) => {
                if let Some(item) = model::get_item_mut(&mut self.inner.data.file, &path) {
                    if let Err(e) = encrypt_item(item, passphrase) {
                        log::warn!("encrypt failed: {e}");
                    }
                }
            }
            CryptoPending::Decrypt(path) => {
                if let Some(item) = model::get_item_mut(&mut self.inner.data.file, &path) {
                    if let Err(e) = decrypt_item(item, passphrase) {
                        log::warn!("decrypt failed: {e}");
                    }
                }
            }
        }
        self.inner.data.save();
        self.inner.data.rebuild_flat();
        self.group.mark_dirty();
    }
}
