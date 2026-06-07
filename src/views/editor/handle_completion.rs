//! EditorView completion popup helpers.

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
        let scroll = self.editor.viewport_scroll();
        let avail = self.text_avail_width();
        let tab_w = self.editor.options().tab_width();

        let mut vis_row: usize = 0;
        for li in scroll..self.editor.cursor_line() {
            vis_row += if self.editor.options().wrap() {
                self.wrapped_line_rows(li, avail)
            } else {
                1
            };
        }
        let (cursor_vrow, cursor_vcol) = self.cursor_visual_pos(
            self.editor.cursor_line(),
            self.editor.cursor_col(),
            avail,
            tab_w,
            vis_row,
        );

        let h_off = if self.editor.options().wrap() {
            0
        } else {
            self.editor.h_scroll()
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
                additional_edits: Vec::new(),
            })
            .collect();
        self.show_completion_items(&items);
    }

    /// Accept the currently selected completion item.
    /// Replaces the entire word under/before cursor with the completion text.
    /// Also applies additionalTextEdits (e.g. auto-imports) maintaining sort order.
    pub(super) fn accept_completion(&mut self) {
        let text = self.completion_popup.selected_text().map(|s| s.to_string());
        let additional = self.completion_popup.selected_additional_edits().to_vec();
        self.completion_popup.hide();
        if let Some(text) = text {
            self.replace_word_with_completion(&text);
            if !additional.is_empty() {
                self.apply_additional_edits(&additional);
            }
        }
        self.clear_diagnostics();
        self.state.mark_dirty();
    }

    fn replace_word_with_completion(&mut self, text: &str) {
        let line = self.editor.buf().line(self.editor.cursor_line()).unwrap_or_default();
        let col = self.editor.cursor_col();
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
            .line_col_to_offset(self.editor.cursor_line(), word_start)
            .unwrap_or(0);
        let end_offset = self
            .editor
            .buf()
            .line_col_to_offset(self.editor.cursor_line(), word_end)
            .unwrap_or(start_offset);
        if end_offset > start_offset {
            self.editor.buf().delete(start_offset, end_offset);
        }
        self.editor.buf().insert(start_offset, text);
        self.editor.set_cursor_col(word_start + text.len());
        self.last_edit_tick = self.tick_counter;
    }

    /// Apply additional text edits from a completion (e.g. auto-imports).
    /// For import insertions, finds the sorted position in the import block.
    fn apply_additional_edits(&mut self, edits: &[crate::lsp::requests::TextEdit]) {
        use crate::lsp::requests::TextEdit;

        let mut sorted: Vec<&TextEdit> = edits.iter().collect();
        sorted.sort_by(|a, b| (b.start_line, b.start_col).cmp(&(a.start_line, a.start_col)));

        for edit in sorted {
            let sl = edit.start_line as usize;
            let sc = edit.start_col as usize;
            let el = edit.end_line as usize;
            let ec = edit.end_col as usize;

            // For pure insertions of import lines, find sorted position
            let (insert_line, insert_col, text) = if sl == el && sc == ec && Self::is_import_line(&edit.new_text) {
                self.find_sorted_import_position(sl, &edit.new_text)
                    .unwrap_or((sl, sc, edit.new_text.clone()))
            } else {
                (sl, sc, edit.new_text.clone())
            };

            let start = self
                .editor
                .buf()
                .line_col_to_offset(insert_line, insert_col)
                .unwrap_or(0);
            let end = self.editor.buf().line_col_to_offset(el, ec).unwrap_or(start);
            if end > start {
                self.editor.buf().delete(start, end - start);
            }
            if !text.is_empty() {
                self.editor.buf().insert(start, &text);
            }
        }
    }

    /// Check if text looks like an import statement.
    fn is_import_line(text: &str) -> bool {
        let t = text.trim();
        t.starts_with("use ")
            || t.starts_with("import ")
            || t.starts_with("from ")
            || t.starts_with("#include")
            || t.starts_with("require")
    }

    /// Find the sorted position for an import line in the import block.
    /// Returns (line, col, text) for where to insert, or None to use the original position.
    fn find_sorted_import_position(&self, insert_at: usize, new_text: &str) -> Option<(usize, usize, String)> {
        // Extract the actual import line content
        let import_trimmed = new_text.trim().trim_end_matches('\n');

        // Find the import block boundaries around the insertion point
        let total_lines = self.editor.buf().line_count();
        let mut block_start = insert_at;
        while block_start > 0 {
            let prev = self.editor.buf().line(block_start - 1).unwrap_or_default();
            if Self::is_import_line(&prev) {
                block_start -= 1;
            } else {
                break;
            }
        }
        let mut block_end = insert_at;
        while block_end < total_lines {
            let line = self.editor.buf().line(block_end).unwrap_or_default();
            if Self::is_import_line(&line) {
                block_end += 1;
            } else {
                break;
            }
        }

        if block_start == block_end {
            return None; // No existing import block, use LSP's position
        }

        // Find sorted position
        for line_idx in block_start..block_end {
            let existing = self.editor.buf().line(line_idx).unwrap_or_default();
            if import_trimmed < existing.trim() {
                // Insert before this line
                return Some((line_idx, 0, format!("{}\n", import_trimmed)));
            }
        }
        // Insert after the last import in the block
        Some((block_end, 0, format!("{}\n", import_trimmed)))
    }

    /// Get the word prefix before the cursor as a string.
    fn word_prefix(&self) -> String {
        let line = self.editor.buf().line(self.editor.cursor_line()).unwrap_or_default();
        let col = self.editor.cursor_col();
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
        let line = self.editor.buf().line(self.editor.cursor_line()).unwrap_or_default();
        let col = self.editor.cursor_col();
        let before = &line[..col.min(line.len())];
        before
            .chars()
            .rev()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .count()
    }

    /// Check if cursor is inside a function call (unmatched open paren before cursor).
    pub(super) fn is_inside_call(&self) -> bool {
        let line = self.editor.buf().line(self.editor.cursor_line()).unwrap_or_default();
        let before = &line[..self.editor.cursor_col().min(line.len())];
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
}
