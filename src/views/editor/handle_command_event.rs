//! Command event dispatch for KairnDelegate.

use txv_core::event::CommandId;
use txv_core::prelude::HandleResult;

use super::delegate::KairnDelegate;
use crate::commands::{
    CM_BLAME, CM_CLIPBOARD_PASTE, CM_DIAGNOSTIC, CM_DIFF, CM_GOTO_LINE, CM_LSP_COMPLETION, CM_LSP_FORMAT_RESULT,
    CM_NOBLAME,
};
use crate::editor::Editor;
use crate::lsp::diagnostics::Diagnostic;
use crate::lsp::requests::CompletionItem;

impl KairnDelegate {
    pub(crate) fn handle_command_event(
        &mut self,
        id: CommandId,
        data: &Option<Box<dyn std::any::Any + Send>>,
        editor: &mut Editor,
    ) -> HandleResult {
        if id == CM_DIFF {
            return self.handle_diff_command(data, editor);
        }
        if id == CM_GOTO_LINE {
            return self.handle_goto_line(data, editor);
        }
        if id == CM_CLIPBOARD_PASTE {
            return self.handle_clipboard_paste(data, editor);
        }
        if id == CM_DIAGNOSTIC {
            return self.handle_diagnostic(data);
        }
        if id == CM_LSP_COMPLETION {
            return self.handle_lsp_completion(data, editor);
        }
        if id == CM_BLAME {
            self.toggle_blame_state();
            return HandleResult::Consumed;
        }
        if id == CM_NOBLAME {
            self.blame_state = None;
            self.dirty = true;
            return HandleResult::Consumed;
        }
        if id == CM_LSP_FORMAT_RESULT {
            return self.handle_format_result(data, editor);
        }
        HandleResult::Ignored
    }

    fn handle_goto_line(&mut self, data: &Option<Box<dyn std::any::Any + Send>>, editor: &mut Editor) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(&(line, col)) = boxed.downcast_ref::<(u32, u32)>() {
                let max = editor.buf().line_count().saturating_sub(1);
                let target = (line as usize).min(max);
                editor.set_cursor_line(target);
                editor.set_cursor_col(col as usize);
                Self::ensure_line_visible(editor, target);
                self.dirty = true;
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_clipboard_paste(
        &mut self,
        data: &Option<Box<dyn std::any::Any + Send>>,
        editor: &mut Editor,
    ) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(text) = boxed.downcast_ref::<String>() {
                let offset = editor
                    .buf()
                    .line_col_to_offset(editor.cursor_line(), editor.cursor_col())
                    .unwrap_or(0);
                editor.buf().insert(offset, text);
                self.last_edit_tick = u64::MAX;
                self.clear_diagnostics();
                self.dirty = true;
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_lsp_completion(&mut self, data: &Option<Box<dyn std::any::Any + Send>>, editor: &Editor) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some(items) = boxed.downcast_ref::<Vec<CompletionItem>>() {
                self.show_completion_items(items, editor);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }

    fn handle_diff_command(
        &mut self,
        data: &Option<Box<dyn std::any::Any + Send>>,
        editor: &mut Editor,
    ) -> HandleResult {
        let args = data
            .as_ref()
            .and_then(|b| b.downcast_ref::<String>())
            .map(|s| s.as_str())
            .unwrap_or("");
        self.enter_diff_mode(editor, args);
        HandleResult::Consumed
    }

    fn handle_diagnostic(&mut self, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        if let Some(boxed) = data.as_ref() {
            if let Some((uri, diags)) = boxed.downcast_ref::<(String, Vec<Diagnostic>)>() {
                let self_path = self.path.to_string_lossy();
                if *uri == self_path.as_ref() {
                    self.diagnostics = Some(diags.clone());
                    self.dirty = true;
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }

    fn handle_format_result(
        &mut self,
        data: &Option<Box<dyn std::any::Any + Send>>,
        editor: &mut Editor,
    ) -> HandleResult {
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
        editor.buf().begin_group();
        for (sl, sc, el, ec, new_text) in &parsed {
            let start = editor.buf().line_col_to_offset(*sl, *sc).unwrap_or(0);
            let end = editor.buf().line_col_to_offset(*el, *ec).unwrap_or(start);
            if end > start {
                editor.buf().delete(start, end - start);
            }
            if !new_text.is_empty() {
                editor.buf().insert(start, new_text);
            }
        }
        editor.buf().end_group();
        editor.clamp_cursor();
        self.dirty = true;
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

impl KairnDelegate {
    pub(super) fn toggle_blame_state(&mut self) {
        use crate::blame::blame_async;
        self.blame_state = if self.blame_state.is_some() {
            None
        } else {
            let root = self.root_dir.clone();
            let rel = self.path.strip_prefix(&root).unwrap_or(&self.path).to_path_buf();
            Some(blame_async(&root, &rel))
        };
        self.dirty = true;
    }
}
