//! TodoTreeView operations — confirm delete, crypto, note updates.

use duir_core::crypto::{decrypt_item, encrypt_item};
use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;

use super::{model, CryptoPending, TodoTreeView};
use crate::commands::{
    CM_TODO_LOE_DOWN, CM_TODO_LOE_UP, CM_TODO_NOTE_UPDATE, CM_TODO_PRIORITY_DOWN, CM_TODO_PRIORITY_UP,
    CM_TODO_TOGGLE_PAUSE, CM_TODO_TOGGLE_PROGRESS,
};
use model::WorkStatus;
use txv_widgets::input_line::CM_CLIPBOARD_PASTE;

impl TodoTreeView {
    pub(super) fn emit_note_update_if_cursor_changed(&mut self, prev_cursor: usize) {
        if self.inner_mut().cursor() == prev_cursor {
            return;
        }
        let cursor = self.inner_mut().cursor();
        if cursor >= self.inner_mut().data_mut().visible_count() {
            return;
        }
        let id = self.inner_mut().data_mut().visible_id(cursor);
        if let Some(path) = self.inner_mut().data_mut().path_at(id) {
            let path = path.clone();
            if let Some(item) = model::get_item(self.inner_mut().data_mut().file(), &path) {
                let note = item.note.clone();
                self.group
                    .put_command(CM_TODO_NOTE_UPDATE, Some(Box::new((path, note))));
            }
        }
    }

    /// Execute the pending delete.
    pub fn confirm_delete_execute(&mut self) {
        let cursor = self.inner_mut().cursor();
        if cursor < self.inner_mut().data_mut().visible_count() {
            let id = self.inner_mut().data_mut().visible_id(cursor);
            if let Some(path) = self.inner_mut().data_mut().path_at(id) {
                let path = path.clone();
                model::remove_item(self.inner_mut().data_mut().file_mut(), &path);
                model::propagate_completion(self.inner_mut().data_mut().file_mut(), &path);
                self.inner_mut().data_mut().save();
                self.inner_mut().data_mut().rebuild_flat();
                self.mark_tree_dirty();
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
                if let Some(item) = model::get_item_mut(self.inner_mut().data_mut().file_mut(), &path) {
                    if let Err(e) = encrypt_item(item, passphrase) {
                        log::warn!("encrypt failed: {e}");
                    }
                }
            }
            CryptoPending::Decrypt(path) => {
                if let Some(item) = model::get_item_mut(self.inner_mut().data_mut().file_mut(), &path) {
                    if let Err(e) = decrypt_item(item, passphrase) {
                        log::warn!("decrypt failed: {e}");
                    }
                }
            }
        }
        self.inner_mut().data_mut().save();
        self.inner_mut().data_mut().rebuild_flat();
        self.mark_tree_dirty();
    }

    /// Handle commands from the status bar FocusGatedGroup.
    pub(super) fn handle_status_command(&mut self, id: CommandId) -> HandleResult {
        let cursor = self.inner_mut().cursor();
        if cursor >= self.inner_mut().data_mut().visible_count() {
            return HandleResult::Ignored;
        }
        let node_id = self.inner_mut().data_mut().visible_id(cursor);
        let Some(path) = self.inner_mut().data_mut().path_at(node_id).cloned() else {
            return HandleResult::Ignored;
        };
        match id {
            CM_TODO_TOGGLE_PROGRESS => self.toggle_progress(&path),
            CM_TODO_TOGGLE_PAUSE => self.toggle_pause(&path),
            CM_TODO_PRIORITY_UP => self.priority_change(&path, 1),
            CM_TODO_PRIORITY_DOWN => self.priority_change(&path, -1),
            CM_TODO_LOE_UP => self.loe_change(&path, true),
            CM_TODO_LOE_DOWN => self.loe_change(&path, false),
            _ => return HandleResult::Ignored,
        }
        self.inner_mut().data_mut().save();
        self.inner_mut().data_mut().rebuild_flat();
        self.mark_tree_dirty();
        HandleResult::Consumed
    }

    fn toggle_progress(&mut self, path: &model::TreePath) {
        let Some(item) = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path) else {
            return;
        };
        if !item.items.is_empty() {
            return; // leaf-only
        }
        item.work_status = match item.work_status {
            WorkStatus::InProgress => WorkStatus::Idle,
            _ => WorkStatus::InProgress,
        };
    }

    fn toggle_pause(&mut self, path: &model::TreePath) {
        let Some(item) = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path) else {
            return;
        };
        if !item.items.is_empty() {
            return; // leaf-only
        }
        item.work_status = match item.work_status {
            WorkStatus::Paused => WorkStatus::Idle,
            _ => WorkStatus::Paused,
        };
    }

    fn priority_change(&mut self, path: &model::TreePath, delta: i8) {
        let Some(item) = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path) else {
            return;
        };
        let current = item.priority.unwrap_or(0);
        let new_val = (i16::from(current) + i16::from(delta)).clamp(0, 9) as u8;
        item.priority = if new_val == 0 {
            None
        } else {
            Some(new_val)
        };
    }

    fn loe_change(&mut self, path: &model::TreePath, up: bool) {
        const FIBONACCI: &[u8] = &[0, 1, 2, 3, 5, 8, 13, 21];
        let Some(item) = model::get_item_mut(self.inner_mut().data_mut().file_mut(), path) else {
            return;
        };
        let current = item.effort.unwrap_or(0);
        if !up && current == 0 {
            // Already at 0 and pressing < — toggle LOE column off if no items have effort.
            if self.inner_mut().data_mut().show_loe()
                && !self.inner_mut().data_mut().loe_strings().iter().any(|s| s.trim() != "")
            {
                self.inner_mut().data_mut().set_show_loe(false);
            }
            return;
        }
        let idx = FIBONACCI.iter().position(|&v| v >= current).unwrap_or(0);
        let new_idx = if up {
            (idx + 1).min(FIBONACCI.len() - 1)
        } else {
            idx.saturating_sub(1)
        };
        let new_val = FIBONACCI[new_idx];
        item.effort = if new_val == 0 {
            None
        } else {
            Some(new_val)
        };
        if !self.inner_mut().data_mut().show_loe() {
            self.inner_mut().data_mut().set_show_loe(true);
        }
    }
    pub(super) fn clipboard_paste(&self) -> Option<String> {
        self.clipboard.as_ref()?.lock().ok()?.paste()
    }

    /// Intercept Ctrl+V in edit mode: paste from ring directly into InputLine.
    pub(super) fn handle_edit_paste(&mut self, event: &Event) -> bool {
        let Event::Key(k) = event else {
            return false;
        };
        if !k.modifiers().ctrl() || k.code() != KeyCode::Char('v') {
            return false;
        }
        let Some(text) = self.clipboard_paste() else {
            return false;
        };
        let paste_ev = Event::Command {
            id: CM_CLIPBOARD_PASTE,
            data: Some(Box::new(text)),
            broadcast: false,
        };
        self.group.dispatch(&paste_ev);
        self.mark_tree_dirty();
        true
    }
}
