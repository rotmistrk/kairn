//! Completion accept helpers — word replacement and prefix computation.

use super::delegate::KairnDelegate;
use crate::editor::Editor;

impl KairnDelegate {
    pub(crate) fn common_completion_prefix(&self, typed: &str) -> Option<String> {
        if self.completion_items.is_empty() {
            return None;
        }
        let lower_typed = typed.to_lowercase();
        let matching: Vec<&str> = self
            .completion_items
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

    pub(crate) fn replace_word_with_completion(&mut self, editor: &mut Editor, text: &str) {
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

    pub(crate) fn apply_additional_edits(&self, editor: &mut Editor, edits: &[crate::lsp::text_edit::TextEdit]) {
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
