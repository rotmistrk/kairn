//! Signature help display for EditorView.

use txv_core::prelude::*;

use super::EditorView;

impl EditorView {
    pub(super) fn show_signature_help(&mut self, sig: &crate::lsp::requests::SignatureHelp) {
        let msg = if let Some(active) = sig.active_param {
            if let Some(&(start, end)) = sig.params.get(active) {
                let label = &sig.label;
                let before = &label[..start.min(label.len())];
                let param = &label[start.min(label.len())..end.min(label.len())];
                let after = &label[end.min(label.len())..];
                format!("{before}[{param}]{after}")
            } else {
                sig.label.clone()
            }
        } else {
            sig.label.clone()
        };
        self.state.put_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("sig", msg))),
        );
    }
}
