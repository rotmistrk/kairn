//! Command event dispatch for EditorView.

use txv_core::message::Message;
use txv_core::prelude::*;

use super::EditorView;
use crate::commands::{
    DiffSplitRequest, CM_BLAME, CM_CLIPBOARD_PASTE, CM_DIAGNOSTIC, CM_DIFF, CM_DIFF_REVERT, CM_DIFF_SPLIT,
    CM_GOTO_LINE, CM_LSP_COMPLETION, CM_LSP_FORMAT_RESULT, CM_LSP_SIGNATURE_HELP, CM_MODE_CHANGED, CM_NOBLAME,
};
use crate::lsp::diagnostics::Diagnostic;
use crate::lsp::protocol::path_to_uri;
use crate::lsp::requests::{CompletionItem, SignatureHelp};

impl EditorView {
    /// Handle Command events dispatched to the editor view.
    pub(super) fn handle_command_event(
        &mut self,
        id: u16,
        data: &Option<Box<dyn std::any::Any + Send>>,
    ) -> HandleResult {
        if id == CM_DIFF {
            return self.handle_diff_command(data);
        }
        if id == CM_BLAME {
            self.toggle_blame();
            return HandleResult::Consumed;
        }
        if id == CM_NOBLAME {
            self.blame_state = None;
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }
        if id == CM_DIFF_REVERT {
            return self.handle_diff_revert_command();
        }
        if id == CM_GOTO_LINE {
            return self.handle_goto_line_command(data);
        }
        if id == CM_CLIPBOARD_PASTE {
            return self.handle_paste_command(data);
        }
        if id == CM_LSP_COMPLETION {
            return self.handle_lsp_completion_command(data);
        }
        if id == CM_LSP_SIGNATURE_HELP {
            return self.handle_lsp_sig_command(data);
        }
        if id == CM_DIAGNOSTIC {
            return self.handle_diagnostic_command(data);
        }
        if id == CM_LSP_FORMAT_RESULT {
            return self.handle_format_result(data);
        }
        HandleResult::Ignored
    }

    fn handle_diff_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        let args = data
            .as_ref()
            .and_then(|b| b.downcast_ref::<String>())
            .map(|s| s.as_str())
            .unwrap_or("");
        if let Some((base_content, base_ref)) = self.try_diff_side_by_side(args) {
            let payload = DiffSplitRequest { base_content, base_ref };
            self.state.put_command(CM_DIFF_SPLIT, Some(Box::new(payload)));
            return HandleResult::Consumed;
        }
        self.toggle_diff(args);
        if !self.editor.status().is_empty() {
            let msg = Message::info("editor", self.editor.status().to_string());
            self.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        let mode = if self.in_diff_mode() {
            "DIFF"
        } else {
            "NOR"
        };
        self.state
            .put_command(CM_MODE_CHANGED, Some(Box::new(mode.to_string())));
        HandleResult::Consumed
    }

    fn handle_diff_revert_command(&mut self) -> HandleResult {
        let msg = match self.revert_hunk() {
            Ok(m) => Message::info("editor", m),
            Err(e) => Message::error("editor", e),
        };
        self.state
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        HandleResult::Consumed
    }

    fn handle_goto_line_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() {
                self.goto(line, col);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_paste_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(text) = boxed.downcast_ref::<String>() {
                let offset = self
                    .editor
                    .buf()
                    .line_col_to_offset(self.editor.cursor_line(), self.editor.cursor_col())
                    .unwrap_or(0);
                self.editor.buf().insert(offset, text);
                self.last_edit_tick = self.tick_counter;
                self.clear_diagnostics();
                self.state.mark_dirty();
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_lsp_completion_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(items) = boxed.downcast_ref::<Vec<CompletionItem>>() {
                self.show_completion_items(items);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_lsp_sig_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(sig) = boxed.downcast_ref::<SignatureHelp>() {
                self.show_signature_help(sig);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_diagnostic_command(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some((uri, diags)) = boxed.downcast_ref::<(String, Vec<Diagnostic>)>() {
                let file_uri = path_to_uri(&self.path);
                if *uri == file_uri {
                    self.set_diagnostics(diags.clone());
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }

    fn handle_format_result(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        let Some(boxed) = data.as_ref() else {
            return HandleResult::Ignored;
        };
        let Some(edits_value) = boxed.downcast_ref::<serde_json::Value>() else {
            return HandleResult::Ignored;
        };
        let Some(edits) = edits_value.as_array() else {
            return HandleResult::Ignored;
        };

        let mut parsed = parse_format_edits(edits);
        if parsed.is_empty() {
            return HandleResult::Consumed;
        }

        parsed.sort_by(|a, b| (b.0, b.1).cmp(&(a.0, a.1)));
        self.editor.buf().begin_group();
        for (sl, sc, el, ec, new_text) in &parsed {
            let start = self.editor.buf().line_col_to_offset(*sl, *sc).unwrap_or(0);
            let end = self.editor.buf().line_col_to_offset(*el, *ec).unwrap_or(start);
            if end > start {
                self.editor.buf().delete(start, end - start);
            }
            if !new_text.is_empty() {
                self.editor.buf().insert(start, new_text);
            }
        }
        self.editor.buf().end_group();
        self.editor.clamp_cursor();
        self.invalidate_highlight();
        self.state.mark_dirty();

        let msg = Message::info("lsp", format!("Formatted ({} edits)", parsed.len()));
        self.state
            .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        HandleResult::Consumed
    }
}

fn parse_format_edits(edits: &[serde_json::Value]) -> Vec<(usize, usize, usize, usize, String)> {
    edits
        .iter()
        .filter_map(|e| {
            let range = e.get("range")?;
            let start = range.get("start")?;
            let end = range.get("end")?;
            let sl = start.get("line")?.as_u64()? as usize;
            let sc = start.get("character")?.as_u64()? as usize;
            let el = end.get("line")?.as_u64()? as usize;
            let ec = end.get("character")?.as_u64()? as usize;
            let new_text = e.get("newText")?.as_str()?.to_string();
            Some((sl, sc, el, ec, new_text))
        })
        .collect()
}
