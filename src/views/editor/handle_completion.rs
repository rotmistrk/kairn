//! EditorView completion popup helpers.

use txv_core::prelude::*;

use crate::lsp::requests::CompletionItem;

use super::EditorView;

impl EditorView {
    /// Show completion popup with labels from LSP response.
    pub(super) fn show_completion(&mut self, labels: &[String]) {
        let items: Vec<CompletionItem> = labels
            .iter()
            .map(|l| CompletionItem {
                label: l.clone(),
                detail: None,
                insert_text: None,
            })
            .collect();
        let b = self.state.bounds();
        let gutter_w = self.gutter_width();
        let scroll = self.editor.viewport_scroll;
        let x = b.x + gutter_w + self.editor.cursor_col as u16;
        let y = b.y + (self.editor.cursor_line - scroll) as u16;
        self.completion_popup.show(items, x, y);
        self.state.mark_dirty();
    }

    /// Accept the currently selected completion item.
    pub(super) fn accept_completion(&mut self) {
        let text = self.completion_popup.selected_text().map(|s| s.to_string());
        self.completion_popup.hide();
        if let Some(text) = text {
            let offset = self
                .editor
                .buffer
                .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                .unwrap_or(0);
            self.editor.buffer.insert(offset, &text);
            self.editor.cursor_col += text.len();
            self.last_edit_tick = self.tick_counter;
        }
        self.state.mark_dirty();
    }

    /// Handle tick event: autosave + completion trigger + LSP didChange.
    pub(super) fn handle_tick(&mut self, queue: &mut EventQueue) {
        self.tick_counter += 1;
        // LSP didChange: 3 ticks after last edit (debounced)
        if self.last_edit_tick > 0 && self.tick_counter - self.last_edit_tick == 3 {
            let changed = crate::commands::ContentChanged {
                path: self.path.clone(),
                content: self.editor.buffer.content(),
            };
            queue.put_command(crate::commands::CM_CONTENT_CHANGED, Some(Box::new(changed)));
        }
        // Completion trigger: 5 ticks after last edit in insert mode
        if self.editor.mode == crate::editor::keymap::EditorMode::Insert
            && self.last_edit_tick > 0
            && self.tick_counter - self.last_edit_tick == 5
        {
            let pos = (self.editor.cursor_line as u32, self.editor.cursor_col as u32);
            queue.put_command(crate::commands::CM_LSP_COMPLETION, Some(Box::new(pos)));
        }
        if self.settings.autosave
            && self.last_edit_tick > 0
            && self.tick_counter - self.last_edit_tick >= self.settings.autosave_delay as u64
        {
            self.last_edit_tick = 0;
            if self.editor.buffer.is_dirty() && self.save_buffer() {
                self.sync_title();
            }
        }
    }

    /// Save buffer to disk. Returns true on success.
    pub(super) fn save_buffer(&mut self) -> bool {
        let content = self.editor.buffer.content();
        if crate::editor::save::save_file(&self.path, &content).is_ok() {
            self.editor.buffer.mark_saved();
            true
        } else {
            false
        }
    }
}
