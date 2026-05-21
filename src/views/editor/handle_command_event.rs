//! Command event dispatch for EditorView.

use crate::commands::CM_CLIPBOARD_PASTE;
use txv_core::prelude::*;

use super::EditorView;

impl EditorView {
    /// Handle Command events dispatched to the editor view.
    pub(super) fn handle_command_event(
        &mut self,
        id: u16,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> HandleResult {
        if id == crate::commands::CM_DIFF {
            let args = data
                .as_ref()
                .and_then(|b| b.downcast_ref::<String>())
                .map(|s| s.as_str())
                .unwrap_or("");
            if let Some((base_content, base_ref)) = self.try_diff_side_by_side(args) {
                let payload = crate::commands::DiffSplitRequest { base_content, base_ref };
                self.state
                    .put_command(crate::commands::CM_DIFF_SPLIT, Some(Box::new(payload)));
                return HandleResult::Consumed;
            }
            self.toggle_diff(args);
            if !self.editor.status.is_empty() {
                let msg = txv_core::message::Message::info("editor", self.editor.status.clone());
                self.state
                    .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
            let mode = if self.in_diff_mode() {
                "DIFF"
            } else {
                "NOR"
            };
            self.state
                .put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new(mode.to_string())));
            return HandleResult::Consumed;
        }
        if id == crate::commands::CM_BLAME {
            self.toggle_blame();
            return HandleResult::Consumed;
        }
        if id == crate::commands::CM_NOBLAME {
            self.blame_state = None;
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }
        if id == crate::commands::CM_DIFF_REVERT {
            let msg = match self.revert_hunk() {
                Ok(m) => txv_core::message::Message::info("editor", m),
                Err(e) => txv_core::message::Message::error("editor", e),
            };
            self.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            return HandleResult::Consumed;
        }
        if id == crate::commands::CM_GOTO_LINE {
            if let Some(boxed) = data.as_ref() {
                if let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() {
                    self.goto(line, col);
                    return HandleResult::Consumed;
                }
            }
        }
        if id == CM_CLIPBOARD_PASTE {
            if let Some(boxed) = data.as_ref() {
                if let Some(text) = boxed.downcast_ref::<String>() {
                    let offset = self
                        .editor
                        .buf()
                        .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                        .unwrap_or(0);
                    self.editor.buf().insert(offset, text);
                    self.last_edit_tick = self.tick_counter;
                    self.clear_diagnostics();
                    self.state.mark_dirty();
                    return HandleResult::Consumed;
                }
            }
        }
        if id == crate::commands::CM_LSP_COMPLETION {
            if let Some(boxed) = data.as_ref() {
                if let Some(items) = boxed.downcast_ref::<Vec<crate::lsp::requests::CompletionItem>>() {
                    self.show_completion_items(items);
                    return HandleResult::Consumed;
                }
            }
        }
        if id == crate::commands::CM_LSP_SIGNATURE_HELP {
            if let Some(boxed) = data.as_ref() {
                if let Some(sig) = boxed.downcast_ref::<crate::lsp::requests::SignatureHelp>() {
                    self.show_signature_help(sig);
                    return HandleResult::Consumed;
                }
            }
        }
        if id == crate::commands::CM_DIAGNOSTIC {
            if let Some(boxed) = data.as_ref() {
                if let Some((uri, diags)) = boxed.downcast_ref::<(String, Vec<crate::lsp::diagnostics::Diagnostic>)>() {
                    let file_uri = format!("file://{}", self.path.display());
                    if *uri == file_uri {
                        self.set_diagnostics(diags.clone());
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}
