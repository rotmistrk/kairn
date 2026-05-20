//! TodoTreeView action dispatch.

use txv_widgets::inline_edit::InlineEditor;
use txv_widgets::tree_view::TreeData;

use super::handle::HandleAction;
use super::{CryptoPending, TodoTreeView};

impl TodoTreeView {
    pub(super) fn apply_action(&mut self, action: HandleAction) {
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
                self.inner.state.put_command(
                    crate::commands::CM_SET_CONFIRM_CONTEXT,
                    Some(Box::new(crate::commands::ConfirmContext::TodoCrypto)),
                );
                self.inner
                    .state
                    .put_command(crate::commands::CM_CONFIRM, Some(Box::new("Passphrase: ".to_string())));
            }
            HandleAction::CryptoDecrypt(path) => {
                self.crypto_pending = Some(CryptoPending::Decrypt(path));
                self.inner.state.put_command(
                    crate::commands::CM_SET_CONFIRM_CONTEXT,
                    Some(Box::new(crate::commands::ConfirmContext::TodoCrypto)),
                );
                self.inner
                    .state
                    .put_command(crate::commands::CM_CONFIRM, Some(Box::new("Passphrase: ".to_string())));
            }
            HandleAction::OpenNote(path, note) => {
                self.inner
                    .state
                    .put_command(crate::commands::CM_TODO_NOTE_OPEN, Some(Box::new((path, note, false))));
            }
            HandleAction::OpenNoteFocus(path, note) => {
                self.inner
                    .state
                    .put_command(crate::commands::CM_TODO_NOTE_OPEN, Some(Box::new((path, note, true))));
            }
        }
        self.inner.mark_dirty();
    }
}
