//! EditorView completion popup helpers.

use txv_core::message::Message;

use crate::lsp::requests::{CompletionItem, CompletionKind};

use super::EditorView;

impl EditorView {
    /// Show completion popup with items from LSP response, filtered by typed prefix.
    pub(super) fn show_completion_items(&mut self, items: &[CompletionItem]) {
        let prefix = self.word_prefix();
        let filtered = self.filter_completion_items(items, &prefix);
        if filtered.is_empty() {
            self.completion_popup.hide();
            self.state.mark_dirty();
            return;
        }
        let (x, y) = self.completion_popup_position(&prefix);
        self.completion_popup.show(filtered, x, y);
        self.state.mark_dirty();
    }

    fn filter_completion_items(&self, items: &[CompletionItem], prefix: &str) -> Vec<CompletionItem> {
        if prefix.is_empty() {
            return items.to_vec();
        }
        let lower_prefix = prefix.to_lowercase();
        items
            .iter()
            .filter(|item| {
                let text = item.insert_text.as_deref().unwrap_or(&item.label);
                text.to_lowercase().starts_with(&lower_prefix)
            })
            .cloned()
            .collect()
    }

    fn completion_popup_position(&self, prefix: &str) -> (u16, u16) {
        let gutter_w = self.gutter_width();
        let scroll = self.editor.viewport_scroll;
        let avail = self.text_avail_width();
        let tab_w = self.editor.options.tab_width;

        let mut vis_row: usize = 0;
        for li in scroll..self.editor.cursor_line {
            vis_row += if self.editor.options.wrap {
                self.wrapped_line_rows(li, avail)
            } else {
                1
            };
        }
        let (cursor_vrow, cursor_vcol) =
            self.cursor_visual_pos(self.editor.cursor_line, self.editor.cursor_col, avail, tab_w, vis_row);

        let h_off = if self.editor.options.wrap {
            0
        } else {
            self.editor.h_scroll
        };
        let screen_col = cursor_vcol.saturating_sub(h_off);
        let x = gutter_w + screen_col.saturating_sub(prefix.len()) as u16;
        let y = cursor_vrow as u16;
        (x, y)
    }

    /// Show completion popup with labels (legacy path).
    pub(super) fn show_completion(&mut self, labels: &[String]) {
        let items: Vec<CompletionItem> = labels
            .iter()
            .map(|l| CompletionItem {
                label: l.clone(),
                detail: None,
                insert_text: None,
                kind: CompletionKind::Other,
            })
            .collect();
        self.show_completion_items(&items);
    }

    /// Accept the currently selected completion item.
    /// Replaces the entire word under/before cursor with the completion text.
    pub(super) fn accept_completion(&mut self) {
        let text = self.completion_popup.selected_text().map(|s| s.to_string());
        self.completion_popup.hide();
        if let Some(text) = text {
            self.replace_word_with_completion(&text);
        }
        self.clear_diagnostics();
        self.state.mark_dirty();
    }

    fn replace_word_with_completion(&mut self, text: &str) {
        let line = self.editor.buf().line(self.editor.cursor_line).unwrap_or_default();
        let col = self.editor.cursor_col;
        let chars: Vec<char> = line.chars().collect();

        let prefix_len = chars[..col]
            .iter()
            .rev()
            .take_while(|c| c.is_alphanumeric() || **c == '_')
            .count();
        let word_start = col - prefix_len;

        let suffix_len = chars[col..]
            .iter()
            .take_while(|c| c.is_alphanumeric() || **c == '_')
            .count();
        let word_end = col + suffix_len;

        let start_offset = self
            .editor
            .buf()
            .line_col_to_offset(self.editor.cursor_line, word_start)
            .unwrap_or(0);
        let end_offset = self
            .editor
            .buf()
            .line_col_to_offset(self.editor.cursor_line, word_end)
            .unwrap_or(start_offset);
        if end_offset > start_offset {
            self.editor.buf().delete(start_offset, end_offset);
        }
        self.editor.buf().insert(start_offset, text);
        self.editor.cursor_col = word_start + text.len();
        self.last_edit_tick = self.tick_counter;
    }

    /// Get the word prefix before the cursor as a string.
    fn word_prefix(&self) -> String {
        let line = self.editor.buf().line(self.editor.cursor_line).unwrap_or_default();
        let col = self.editor.cursor_col;
        let before = &line[..col.min(line.len())];
        before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get the length of the word prefix before the cursor (identifier chars).
    fn word_prefix_len(&self) -> usize {
        let line = self.editor.buf().line(self.editor.cursor_line).unwrap_or_default();
        let col = self.editor.cursor_col;
        let before = &line[..col.min(line.len())];
        before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .count()
    }

    /// Check if cursor is inside a function call (unmatched open paren before cursor).
    pub(super) fn is_inside_call(&self) -> bool {
        let line = self.editor.buf().line(self.editor.cursor_line).unwrap_or_default();
        let before = &line[..self.editor.cursor_col.min(line.len())];
        let mut depth: i32 = 0;
        for ch in before.chars() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
        depth > 0
    }

    /// Display signature help — show function signature in status bar.
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
