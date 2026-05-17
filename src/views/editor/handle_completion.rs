//! EditorView completion popup helpers.

use crate::lsp::requests::CompletionItem;

use super::EditorView;

impl EditorView {
    /// Show completion popup with items from LSP response.
    pub(super) fn show_completion_items(&mut self, items: &[CompletionItem]) {
        let b = self.state.bounds();
        let gutter_w = self.gutter_width();
        let scroll = self.editor.viewport_scroll;
        let x = b.x + gutter_w + self.editor.cursor_col as u16;
        let y = b.y + (self.editor.cursor_line - scroll) as u16;
        self.completion_popup.show(items.to_vec(), x, y);
        self.state.mark_dirty();
    }

    /// Show completion popup with labels (legacy path).
    pub(super) fn show_completion(&mut self, labels: &[String]) {
        let items: Vec<CompletionItem> = labels
            .iter()
            .map(|l| CompletionItem {
                label: l.clone(),
                detail: None,
                insert_text: None,
            })
            .collect();
        self.show_completion_items(&items);
    }

    /// Accept the currently selected completion item.
    /// Removes the word prefix before cursor, then inserts the completion text.
    pub(super) fn accept_completion(&mut self) {
        let text = self.completion_popup.selected_text().map(|s| s.to_string());
        self.completion_popup.hide();
        if let Some(text) = text {
            // Find the word prefix before cursor to replace
            let prefix_len = self.word_prefix_len();
            let offset = self
                .editor
                .buf()
                .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                .unwrap_or(0);
            // Delete the prefix
            if prefix_len > 0 {
                let start = offset - prefix_len;
                self.editor.buf().delete(start, prefix_len);
                self.editor.cursor_col -= prefix_len;
            }
            // Insert completion text
            let insert_offset = offset - prefix_len;
            self.editor.buf().insert(insert_offset, &text);
            self.editor.cursor_col += text.len();
            self.last_edit_tick = self.tick_counter;
        }
        self.state.mark_dirty();
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

    /// Handle tick event: autosave + completion trigger + LSP didChange.
    pub(super) fn handle_tick(&mut self) {
        self.tick_counter += 1;
        // LSP didChange: 3 ticks after last edit (debounced)
        if self.last_edit_tick > 0 && self.tick_counter - self.last_edit_tick == 3 {
            let changed = crate::commands::ContentChanged {
                path: self.path.clone(),
                content: self.editor.buf().content(),
            };
            self.state
                .put_command(crate::commands::CM_CONTENT_CHANGED, Some(Box::new(changed)));
        }
        // Completion trigger: 5 ticks after last edit in insert mode
        if self.editor.mode == crate::editor::keymap::EditorMode::Insert
            && self.last_edit_tick > 0
            && self.tick_counter - self.last_edit_tick == 5
        {
            let pos = (
                self.path.clone(),
                self.editor.cursor_line as u32,
                self.editor.cursor_col as u32,
            );
            self.state
                .put_command(crate::commands::CM_LSP_COMPLETION, Some(Box::new(pos)));
        }
        if self.settings.autosave
            && self.last_edit_tick > 0
            && self.tick_counter - self.last_edit_tick >= self.settings.autosave_delay as u64
        {
            self.last_edit_tick = 0;
            if self.editor.buf().is_dirty() && self.save_buffer() {
                self.sync_title();
            }
        }
    }

    /// Save buffer to disk. Returns true on success.
    /// Save buffer via the configured store. Returns true on success.
    pub(super) fn save_buffer(&mut self) -> bool {
        let content = self.editor.buf().content();
        if self.store.save(&content).is_ok() {
            self.editor.buf().mark_saved();
            true
        } else {
            false
        }
    }

    pub(super) fn complete_command_buf(&mut self) {
        let buf = &self.editor.command_buf;

        // File path completion for :e / :edit
        if buf.starts_with("e ") || buf.starts_with("edit ") {
            self.complete_command_path();
            return;
        }

        // Command name completion
        use crate::editor::ex_commands::CMD_TABLE_NAMES;
        let matches: Vec<&str> = CMD_TABLE_NAMES
            .iter()
            .filter(|cmd| cmd.starts_with(buf.as_str()))
            .copied()
            .collect();
        if matches.len() == 1 {
            self.editor.command_buf = matches[0].to_string();
        }
    }

    fn complete_command_path(&mut self) {
        use std::path::Path;
        let buf = &self.editor.command_buf;
        let partial = buf
            .strip_prefix("e ")
            .or_else(|| buf.strip_prefix("edit "))
            .unwrap_or("");
        let (search_dir, file_prefix, dir_prefix) = if partial.contains('/') {
            let p = Path::new(partial);
            let parent = p.parent().unwrap_or(Path::new(""));
            let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let dp = format!("{}/", parent.display());
            (self.root_dir.join(parent), prefix.to_string(), dp)
        } else {
            (self.root_dir.clone(), partial.to_string(), String::new())
        };

        let Ok(entries) = std::fs::read_dir(&search_dir) else {
            return;
        };
        let mut matches: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if name_str.starts_with(&file_prefix) {
                matches.push(format!("{dir_prefix}{name_str}"));
            }
        }
        if matches.len() == 1 {
            let prefix = if buf.starts_with("edit ") {
                "edit "
            } else {
                "e "
            };
            self.editor.command_buf = format!("{prefix}{}", matches[0]);
        }
    }
}
