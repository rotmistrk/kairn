//! TodoTreeView action dispatch.

use txv_widgets::tree_view::TreeData;

use crate::commands::{ConfirmContext, CM_CONFIRM, CM_SET_CONFIRM_CONTEXT, CM_TODO_NOTE_OPEN};
use txv_widgets::input_line::{CM_COPY_TO_CLIPBOARD, CM_PASTE_REQUEST};

use super::handle::HandleAction;
use super::{CryptoPending, TodoTreeView};

impl TodoTreeView {
    pub(super) fn apply_action(&mut self, action: HandleAction) {
        match action {
            HandleAction::Stay => {}
            HandleAction::MoveDown => {
                let max = self.inner.data.visible_count().saturating_sub(1);
                if self.inner.cursor < max {
                    self.inner.set_cursor(self.inner.cursor + 1);
                }
            }
            HandleAction::MoveTo(row) => {
                self.inner.set_cursor(row);
            }
            HandleAction::EditNew(row) => {
                self.inner.set_cursor(row);
                self.start_edit_selected();
            }
            HandleAction::ConfirmDelete => {}
            HandleAction::EnterFilter => {
                self.start_filter();
            }
            HandleAction::CryptoEncrypt(path) => self.start_crypto(CryptoPending::Encrypt(path)),
            HandleAction::CryptoDecrypt(path) => self.start_crypto(CryptoPending::Decrypt(path)),
            HandleAction::OpenNote(path, note) => {
                self.group
                    .put_command(CM_TODO_NOTE_OPEN, Some(Box::new((path, note, false))));
            }
            HandleAction::OpenNoteFocus(path, note) => {
                self.group
                    .put_command(CM_TODO_NOTE_OPEN, Some(Box::new((path, note, true))));
            }
            HandleAction::CopyToClipboard(text) => {
                self.group.put_command(CM_COPY_TO_CLIPBOARD, Some(Box::new(text)));
            }
            HandleAction::PasteFromClipboard => {
                self.group.put_command(CM_PASTE_REQUEST, None);
            }
        }
        self.group.mark_dirty();
    }

    fn start_crypto(&mut self, pending: CryptoPending) {
        self.crypto_pending = Some(pending);
        self.group
            .put_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ConfirmContext::TodoCrypto)));
        self.group
            .put_command(CM_CONFIRM, Some(Box::new("Passphrase: ".to_string())));
    }
}
