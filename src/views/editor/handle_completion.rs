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
            return self.handle_active_completion(key, editor);
        }
        if key.modifiers().ctrl() && key.code() == KeyCode::Char('n') {
            self.trigger_completion(editor);
            return Some(HandleResult::Consumed);
        }
        None
    }

    fn handle_active_completion(&mut self, key: &KeyEvent, editor: &mut Editor) -> Option<HandleResult> {
        match key.code() {
            KeyCode::Down => {
                self.completion_selected = (self.completion_selected + 1) % self.completion_items.len();
                self.emit(CM_SIDEKICK_NEXT, None);
                Some(HandleResult::Consumed)
            }
            KeyCode::Up => {
                let len = self.completion_items.len();
                self.completion_selected = (self.completion_selected + len - 1) % len;
                self.emit(CM_SIDEKICK_PREV, None);
                Some(HandleResult::Consumed)
            }
            KeyCode::Tab | KeyCode::Right | KeyCode::Enter => {
                self.accept_completion(editor);
                Some(HandleResult::Consumed)
            }
            KeyCode::Esc => {
                self.hide_completion();
                Some(HandleResult::Consumed)
            }
            KeyCode::Char('n') if key.modifiers().ctrl() => {
                self.completion_selected = (self.completion_selected + 1) % self.completion_items.len();
                self.emit(CM_SIDEKICK_NEXT, None);
                Some(HandleResult::Consumed)
            }
            KeyCode::Char('p') if key.modifiers().ctrl() => {
                let len = self.completion_items.len();
                self.completion_selected = (self.completion_selected + len - 1) % len;
                self.emit(CM_SIDEKICK_PREV, None);
                Some(HandleResult::Consumed)
            }
            _ => None, // pass through to editor
        }
    }

    fn trigger_completion(&mut self, editor: &Editor) {
        self.emit(
            CM_LSP_COMPLETION,
            Some(Box::new((
                self.path.clone(),
                editor.cursor_line() as u32,
                editor.cursor_col() as u32,
            ))),
        );
    }

    /// Re-filter completion based on current word prefix. Auto-hide if 0 matches.
    pub(crate) fn refilter_completion(&mut self, editor: &Editor) {
        if !self.completion_visible {
            return;
        }
        let prefix = Self::word_prefix(editor);
        let lp = prefix.to_lowercase();
        let filtered: Vec<_> = self
            .completion_items
            .iter()
            .enumerate()
            .filter(|(_, i)| {
                let t = i.insert_text.as_deref().unwrap_or(&i.label);
                t.to_lowercase().starts_with(&lp)
            })
            .map(|(idx, _)| idx)
            .collect();
        if filtered.is_empty() {
            self.hide_completion();
        } else {
            // Update the sidekick dropdown with filtered view
            let items: Vec<_> = filtered.iter().map(|&i| self.completion_items[i].clone()).collect();
            self.completion_selected = 0;
            self.show_sidekick(items, editor);
        }
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
        self.completion_selected = 0;
        self.show_sidekick(filtered, editor);
        self.dirty = true;
    }

    fn show_sidekick(&mut self, items: Vec<CompletionItem>, editor: &Editor) {
        use txv_edit::view::draw::compute_gutter_width;
        use txv_widgets::dropdown_menu::{DropdownMenu, FilterMode, NumberMode, OpenSide};
        use txv_widgets::sidekick::SidekickRequest;

        let count = items.len();
        let max_label = items.iter().map(|i| i.label.len()).max().unwrap_or(10);
        let max_detail = items
            .iter()
            .filter_map(|i| i.detail.as_ref())
            .map(|d| d.len())
            .max()
            .unwrap_or(0);
        let w = if max_detail > 0 {
            (max_label + max_detail + 4) as u16
        } else {
            max_label as u16 + 4
        }
        .clamp(20, 60);
        let source = LspCompletionSource::new(items);
        let menu = DropdownMenu::new(source)
            .with_numbers(NumberMode::None)
            .with_filter(FilterMode::Prefix)
            .with_open_side(OpenSide::None);
        let content_h = count.min(8) as u16;
        let h = content_h + 2;
        let gw = compute_gutter_width(editor, self);
        let cx = gw + editor.cursor_col().saturating_sub(editor.h_scroll()) as u16;
        let cy = self.visual_cursor_row(editor, gw);
        let rect = Rect::new(cx, cy, w, h);
        let data = SidekickRequest::new(Box::new(menu), rect, self.view_id);
        self.emit(CM_SIDEKICK_SHOW, Some(Box::new(data)));
    }

    /// Compute visual row of cursor accounting for wrapping and sticky lines.
    fn visual_cursor_row(&self, editor: &Editor, gw: u16) -> u16 {
        use txv_core::text::display_width;
        use txv_edit::view::draw::sticky::sticky_line_count;

        let scroll = editor.viewport_scroll();
        let line = editor.cursor_line();
        let sticky_h = sticky_line_count(editor);

        if !editor.options().wrap() {
            return (line.saturating_sub(scroll) as u16) + sticky_h;
        }
        let avail = (editor.viewport_width() as u16).saturating_sub(gw) as usize;
        if avail == 0 {
            return sticky_h;
        }
        let tw = editor.options().tab_width();
        let mut vrow = 0u16;
        for i in scroll..line {
            let l = editor.buf().line(i).unwrap_or_default();
            let w = display_width(&l, tw) as usize;
            vrow += if w == 0 {
                1
            } else {
                w.div_ceil(avail) as u16
            };
        }
        vrow + sticky_h
    }

    fn hide_completion(&mut self) {
        if self.completion_visible {
            self.completion_visible = false;
            self.completion_selected = 0;
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

    pub(crate) fn accept_completion(&mut self, editor: &mut Editor) {
        let prefix = Self::word_prefix(editor);
        if let Some(common) = self.common_completion_prefix(&prefix) {
            if common.len() > prefix.len() {
                self.replace_word_with_completion(editor, &common);
                self.dirty = true;
                return;
            }
        }
        // Common prefix equals typed prefix — accept selected item
        let idx = self.completion_selected;
        let text = self
            .completion_items
            .get(idx)
            .map(|i| i.insert_text.as_deref().unwrap_or(&i.label).to_string());
        let edits = self
            .completion_items
            .get(idx)
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
}
