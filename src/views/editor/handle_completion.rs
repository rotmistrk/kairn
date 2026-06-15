//! Completion handling via DropdownMenu sidekick.

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::prelude::*;
use txv_widgets::sidekick::{CM_SIDEKICK_HIDE, CM_SIDEKICK_NEXT, CM_SIDEKICK_PREV, CM_SIDEKICK_SHOW};

use super::delegate::KairnDelegate;
use crate::commands::CM_LSP_COMPLETION;
use crate::editor::Editor;
use crate::lsp::completion_source::LspCompletionSource;
use crate::lsp::requests::CompletionItem;

impl KairnDelegate {
    pub(crate) fn handle_completion_key(&mut self, key: &KeyEvent, editor: &mut Editor) -> Option<HandleResult> {
        if self.completion_visible {
            match (key.code(), key.modifiers().ctrl()) {
                (KeyCode::Down, _) | (KeyCode::Char('n'), true) => {
                    self.emit(CM_SIDEKICK_NEXT, None);
                    return Some(HandleResult::Consumed);
                }
                (KeyCode::Up, _) | (KeyCode::Char('p'), true) => {
                    self.emit(CM_SIDEKICK_PREV, None);
                    return Some(HandleResult::Consumed);
                }
                (KeyCode::Tab, _) => {
                    self.accept_completion(editor);
                    return Some(HandleResult::Consumed);
                }
                (KeyCode::Enter, _) => {
                    self.hide_completion();
                }
                (KeyCode::Esc, _) => {
                    self.hide_completion();
                    return Some(HandleResult::Consumed);
                }
                _ => {
                    self.hide_completion();
                }
            }
        } else if key.modifiers().ctrl() && key.code() == KeyCode::Char('n') {
            self.emit(
                CM_LSP_COMPLETION,
                Some(Box::new((
                    self.path.clone(),
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
            self.hide_completion();
            return;
        }
        self.completion_items = filtered.clone();
        self.completion_visible = true;
        self.show_sidekick(filtered);
        self.dirty = true;
    }

    fn show_sidekick(&mut self, items: Vec<CompletionItem>) {
        use txv_widgets::dropdown_menu::{DropdownMenu, FilterMode, NumberMode, OpenSide};
        use txv_widgets::sidekick::SidekickRequest;

        let count = items.len();
        let max_w = items.iter().map(|i| i.label.len()).max().unwrap_or(10);
        let source = LspCompletionSource::new(items);
        let menu = DropdownMenu::new(source)
            .with_numbers(NumberMode::None)
            .with_filter(FilterMode::None)
            .with_open_side(OpenSide::None);
        let content_h = count.min(8) as u16;
        let h = content_h + 2;
        let w = (max_w as u16 + 6).clamp(14, 50);
        let rect = Rect::new(0, 0, w, h);
        let data = SidekickRequest::new(Box::new(menu), rect, self.view_id);
        self.emit(CM_SIDEKICK_SHOW, Some(Box::new(data)));
    }

    fn hide_completion(&mut self) {
        if self.completion_visible {
            self.completion_visible = false;
            self.completion_items.clear();
            self.emit(CM_SIDEKICK_HIDE, None);
            self.dirty = true;
        }
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

    fn accept_completion(&mut self, editor: &mut Editor) {
        let prefix = Self::word_prefix(editor);
        if let Some(common) = self.common_completion_prefix(&prefix) {
            if common.len() > prefix.len() {
                self.replace_word_with_completion(editor, &common);
                self.dirty = true;
                return;
            }
        }
        // Common prefix equals typed prefix — accept first item (or selected)
        let text = self
            .completion_items
            .first()
            .map(|i| i.insert_text.as_deref().unwrap_or(&i.label).to_string());
        let edits = self
            .completion_items
            .first()
            .map(|i| i.additional_edits.clone())
            .unwrap_or_default();
        self.hide_completion();
        if let Some(text) = text {
            self.replace_word_with_completion(editor, &text);
        }
        self.apply_additional_edits(editor, &edits);
        self.clear_diagnostics();
        self.dirty = true;
    }

    fn common_completion_prefix(&self, typed: &str) -> Option<String> {
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
