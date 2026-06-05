//! EditorView utility methods (save, settings, navigation, undo/redo).

use super::EditorView;
use crate::blame::blame_async;
use crate::gutter_signs::compute_gutter_signs;

impl EditorView {
    /// Recompute git gutter signs (diff vs HEAD).
    pub fn refresh_gutter_signs(&mut self) {
        if !self.editor.options.gutter_signs {
            self.gutter_signs.clear();
            return;
        }
        let rel = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();
        let content = self.editor.buf().content();
        self.gutter_signs = compute_gutter_signs(&self.root_dir, &rel, &content);
    }

    /// Save the buffer to disk immediately. Returns true on success.
    pub fn save_now(&mut self) -> bool {
        self.save_buffer()
    }

    pub fn language(&self) -> &str {
        &self.file_ext
    }

    pub(super) fn apply_settings(&mut self) {
        self.editor.options.wrap = self.settings.wrap;
        self.editor.options.list = self.settings.list;
        self.editor.options.tab_width = self.settings.tabstop as usize;
        self.editor.options.number = self.settings.number;
        self.editor.options.rainbow = self.settings.rainbow;
        self.editor.options.guides = self.settings.guides;
        self.editor.options.gutter_signs = self.settings.gutter_signs;
        self.editor.options.scrolloff = self.settings.scrolloff;
        self.editor.options.cursor_insert = self.settings.cursor_insert;
        self.editor.options.cursor_normal = self.settings.cursor_normal;
        self.editor.options.cursor_command = self.settings.cursor_command;
    }

    pub fn set_syntax_theme(&mut self, name: &str) {
        self.highlighter.set_theme(name);
        self.hl_cache.borrow_mut().invalidate_all();
    }

    /// Invalidate highlight cache and mark view dirty (for external content reload).
    pub fn invalidate_highlight(&mut self) {
        self.hl_cache.borrow_mut().invalidate_all();
        self.state.mark_dirty();
    }

    /// Replace the persistence backend.
    pub fn set_store(&mut self, store: Box<dyn crate::buffer_store::BufferStore>) {
        self.store = store;
    }

    pub fn save(&mut self) -> Result<(), String> {
        let content = self.editor.buf().content();
        self.store.save(&content)?;
        self.editor.buf().mark_saved();
        Ok(())
    }

    pub fn request_close(&mut self) {
        if self.editor.buf().is_dirty() && !self.settings.autosave {
            self.eviction_close = true;
            self.state.mark_dirty();
        }
    }

    pub fn goto(&mut self, line: u32, col: u32) {
        use crate::editor::ephemeral::HighlightOwner;
        use crate::editor::ephemeral_range::EphemeralRange;
        let max_line = self.editor.buf().line_count().saturating_sub(1);
        let target_line = (line as usize).min(max_line);
        self.editor.cursor_line = target_line;
        self.editor.cursor_col = col as usize;
        self.editor
            .ephemeral
            .set(vec![EphemeralRange::full_line(target_line)], HighlightOwner::Transient);
        self.ensure_cursor_visible();
        if self.state.bounds().h == 0 {
            self.editor.viewport_scroll = self.editor.cursor_line;
        }
        self.state.mark_dirty();
    }

    /// Undo the last edit.
    pub fn undo(&mut self) {
        self.editor.buf().undo();
        self.editor.clamp_cursor();
        self.state.mark_dirty();
    }

    /// Redo the last undone edit.
    pub fn redo(&mut self) {
        self.editor.buf().redo();
        self.editor.clamp_cursor();
        self.state.mark_dirty();
    }

    pub fn gutter_width(&self) -> u16 {
        if !self.editor.options.number {
            return 0;
        }
        let lines = self.editor.buf().line_count();
        let digits = if lines == 0 {
            1
        } else {
            (lines as f64).log10() as u16 + 1
        };
        let sign_w: u16 = if self.editor.options.gutter_signs {
            1
        } else {
            0
        };
        let blame_w = if self.blame_state.is_some() {
            24
        } else {
            0
        };
        sign_w + digits + 1 + blame_w
    }

    /// Toggle blame mode on/off.
    pub(super) fn toggle_blame(&mut self) {
        if self.blame_state.is_some() {
            self.blame_state = None;
        } else {
            let rel = self
                .path
                .strip_prefix(&self.root_dir)
                .unwrap_or(&self.path)
                .to_path_buf();
            self.blame_state = Some(blame_async(&self.root_dir, &rel));
        }
        self.state.mark_dirty();
    }
}
