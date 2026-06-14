//! Completion popup handling in on_key_pre.

use std::path::PathBuf;

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::prelude::HandleResult;
use txv_edit::view::draw::compute_gutter_width;

use super::delegate::KairnDelegate;
use crate::commands::CM_LSP_COMPLETION;
use crate::editor::Editor;
use crate::lsp::requests::CompletionItem;

impl KairnDelegate {
    pub(crate) fn handle_completion_key(&mut self, key: &KeyEvent, editor: &mut Editor) -> Option<HandleResult> {
        if self.completion_popup.visible {
            match (key.code(), key.modifiers().ctrl()) {
                (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
                    self.completion_popup.next();
                    self.dirty = true;
                    return Some(HandleResult::Consumed);
                }
                (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
                    self.completion_popup.prev();
                    self.dirty = true;
                    return Some(HandleResult::Consumed);
                }
                (KeyCode::Tab, _) => {
                    self.accept_completion(editor);
                    return Some(HandleResult::Consumed);
                }
                (KeyCode::Enter, _) => {
                    self.completion_popup.hide();
                }
                (KeyCode::Esc, _) => {
                    self.completion_popup.hide();
                    self.dirty = true;
                    return Some(HandleResult::Consumed);
                }
                _ => {
                    self.completion_popup.hide();
                }
            }
        } else if key.modifiers().ctrl() && key.code() == KeyCode::Char('n') {
            self.emit(
                CM_LSP_COMPLETION,
                Some(Box::new((
                    PathBuf::new(), // will be filled by flush_pending
                    editor.cursor_line() as u32,
                    editor.cursor_col() as u32,
                ))),
            );
            return Some(HandleResult::Consumed);
        }
        None
    }

    pub(crate) fn show_completion_items(&mut self, items: &[CompletionItem], editor: &Editor) {
        let prefix = Self::word_prefix(editor);
        let filtered: Vec<_> = if prefix.is_empty() {
            items.to_vec()
        } else {
            let lp = prefix.to_lowercase();
            items
                .iter()
                .filter(|i| {
                    let t = i.insert_text.as_deref().unwrap_or(&i.label);
                    t.to_lowercase().starts_with(&lp)
                })
                .cloned()
                .collect()
        };
        if filtered.is_empty() {
            self.completion_popup.hide();
        } else {
            let (x, y) = self.popup_position(editor, &prefix);
            self.completion_popup.show(filtered, x, y);
        }
        self.dirty = true;
    }

    fn word_prefix(editor: &Editor) -> String {
        let line = editor.buf().line(editor.cursor_line()).unwrap_or_default();
        let col = editor.cursor_col().min(line.len());
        line[..col]
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    fn popup_position(&self, editor: &Editor, prefix: &str) -> (u16, u16) {
        let scroll = editor.viewport_scroll();
        let line = editor.cursor_line();
        let y = if line >= scroll {
            (line - scroll) as u16
        } else {
            0
        };
        let gw = compute_gutter_width(editor, self);
        let col = editor.cursor_col().saturating_sub(prefix.len());
        let x = gw + col as u16;
        (x, y)
    }

    fn accept_completion(&mut self, editor: &mut Editor) {
        let prefix = Self::word_prefix(editor);
        if let Some(common) = self.common_completion_prefix(&prefix) {
            if common.len() > prefix.len() {
                // Extend to common prefix without accepting any single item
                self.replace_word_with_completion(editor, &common);
                self.dirty = true;
                return;
            }
        }
        // Common prefix equals typed prefix — accept selected item
        let text = self.completion_popup.selected_text().map(|s| s.to_string());
        let edits = self.completion_popup.selected_additional_edits().to_vec();
        self.completion_popup.hide();
        if let Some(text) = text {
            self.replace_word_with_completion(editor, &text);
        }
        self.apply_additional_edits(editor, &edits);
        self.clear_diagnostics();
        self.dirty = true;
    }

    fn common_completion_prefix(&self, typed: &str) -> Option<String> {
        let items = &self.completion_popup.items;
        if items.is_empty() {
            return None;
        }
        let lower_typed = typed.to_lowercase();
        let matching: Vec<&str> = items
            .iter()
            .map(|i| i.insert_text.as_deref().unwrap_or(&i.label))
            .filter(|t| t.to_lowercase().starts_with(&lower_typed))
            .collect();
        if matching.is_empty() {
            return None;
        }
        let first = matching[0];
        let common_len = (0..first.len())
            .take_while(|&i| {
                let ch = first.as_bytes().get(i);
                matching.iter().all(|m| m.as_bytes().get(i) == ch)
            })
            .count();
        Some(first[..common_len].to_string())
    }

    fn replace_word_with_completion(&mut self, editor: &mut Editor, text: &str) {
        let line = editor.buf().line(editor.cursor_line()).unwrap_or_default();
        let col = editor.cursor_col();
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
        let start_offset = editor
            .buf()
            .line_col_to_offset(editor.cursor_line(), word_start)
            .unwrap_or(0);
        let end_offset = editor
            .buf()
            .line_col_to_offset(editor.cursor_line(), word_end)
            .unwrap_or(start_offset);
        if end_offset > start_offset {
            editor.buf().delete(start_offset, end_offset);
        }
        editor.buf().insert(start_offset, text);
        editor.set_cursor_col(word_start + text.len());
        self.last_edit_tick = u64::MAX;
    }

    fn apply_additional_edits(&self, editor: &mut Editor, edits: &[crate::lsp::text_edit::TextEdit]) {
        // Apply edits in reverse order (bottom to top) to preserve offsets
        let mut sorted: Vec<_> = edits.to_vec();
        sorted.sort_by(|a, b| (b.start_line, b.start_col).cmp(&(a.start_line, a.start_col)));
        for edit in &sorted {
            let start = editor
                .buf()
                .line_col_to_offset(edit.start_line as usize, edit.start_col as usize)
                .unwrap_or(0);
            let end = editor
                .buf()
                .line_col_to_offset(edit.end_line as usize, edit.end_col as usize)
                .unwrap_or(start);
            if end > start {
                editor.buf().delete(start, end);
            }
            if !edit.new_text.is_empty() {
                editor.buf().insert(start, &edit.new_text);
            }
        }
    }
}
