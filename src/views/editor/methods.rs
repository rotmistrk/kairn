//! EditorView utility methods.

use std::fs::metadata;
use std::path::PathBuf;

use txv_core::prelude::*;
use txv_edit::view::draw::compute_gutter_width;

use super::EditorView;
use crate::blame::blame_async;
use crate::gutter_signs::compute_gutter_signs;

impl EditorView {
    pub fn refresh_gutter_signs(&mut self) {
        let d = self.inner.delegate();
        if !d.settings.gutter_signs {
            self.inner.delegate_mut().gutter_signs.clear();
            return;
        }
        let root = d.root_dir.clone();
        let rel = self
            .path()
            .strip_prefix(&root)
            .unwrap_or(self.path())
            .to_string_lossy()
            .to_string();
        let content = self.editor().buf().content();
        self.inner.delegate_mut().gutter_signs = compute_gutter_signs(&root, &rel, &content);
    }

    pub fn save_now(&mut self) -> bool {
        self.save_buffer()
    }

    pub(crate) fn save_buffer(&mut self) -> bool {
        let content = self.editor().buf().content();
        if self.inner.delegate_mut().store.save(&content).is_ok() {
            self.editor_mut().buf().mark_saved();
            self.inner.delegate_mut().disk_mtime = metadata(self.path()).and_then(|m| m.modified()).ok();
            self.refresh_gutter_signs();
            true
        } else {
            false
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        let content = self.editor().buf().content();
        self.inner.delegate_mut().store.save(&content)?;
        self.editor_mut().buf().mark_saved();
        Ok(())
    }

    pub fn language(&self) -> &str {
        &self.inner.delegate().file_ext
    }

    pub fn apply_settings(&mut self) {
        let s = self.inner.delegate().settings.clone();
        let opts = self.editor_mut().options_mut();
        opts.set_wrap(s.wrap);
        opts.set_list(s.list);
        opts.set_tab_width(s.tabstop as usize);
        opts.set_number(s.number);
        opts.set_rainbow(s.rainbow);
        opts.set_guides(s.guides);
        opts.set_gutter_signs(s.gutter_signs);
        opts.set_scrolloff(s.scrolloff);
        opts.set_cursor_insert(s.cursor_insert);
        opts.set_cursor_normal(s.cursor_normal);
        opts.set_cursor_command(s.cursor_command);
    }

    pub fn set_syntax_theme(&mut self, name: &str) {
        self.inner.highlighter_mut().set_theme(name);
        self.inner.hl_cache_mut().invalidate_all();
        self.inner.mark_dirty();
    }

    pub fn invalidate_highlight(&mut self) {
        self.inner.hl_cache_mut().invalidate_all();
        self.inner.mark_dirty();
    }

    pub fn set_store(&mut self, store: Box<dyn crate::buffer_store::BufferStore>) {
        self.inner.delegate_mut().store = store;
    }

    pub fn set_root_dir(&mut self, root: PathBuf) {
        self.inner.delegate_mut().root_dir = root;
        self.refresh_gutter_signs();
    }

    pub fn request_close(&mut self) {
        if !self.inner.delegate().settings.autosave {
            self.inner.delegate_mut().eviction_close = true;
        }
        self.inner.mark_dirty();
    }

    pub fn goto(&mut self, line: u32, col: u32) {
        use crate::editor::ephemeral::HighlightOwner;
        use crate::editor::ephemeral_range::EphemeralRange;
        let max_line = self.editor().buf().line_count().saturating_sub(1);
        let target_line = (line as usize).min(max_line);
        self.editor_mut().set_cursor_line(target_line);
        self.editor_mut().set_cursor_col(col as usize);
        self.editor_mut()
            .ephemeral_mut()
            .set(vec![EphemeralRange::full_line(target_line)], HighlightOwner::Transient);
        if self.inner.bounds().h() == 0 {
            self.editor_mut().set_viewport_scroll(target_line);
        }
        self.inner.mark_dirty();
    }

    pub fn undo(&mut self) {
        self.editor_mut().buf().undo();
        self.editor_mut().clamp_cursor();
        self.inner.mark_dirty();
    }

    pub fn redo(&mut self) {
        self.editor_mut().buf().redo();
        self.editor_mut().clamp_cursor();
        self.inner.mark_dirty();
    }

    pub fn sync_title(&mut self) {
        let name = self
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        self.inner.delegate_mut().display_title = name;
    }

    pub fn set_file_ext(&mut self, ext: &str) {
        self.inner.delegate_mut().file_ext = ext.to_string();
    }

    pub fn set_display_title(&mut self, title: &str) {
        self.inner.delegate_mut().display_title = title.to_string();
    }

    pub fn gutter_width(&self) -> u16 {
        compute_gutter_width(self.editor(), self.delegate())
    }

    pub fn toggle_blame(&mut self) {
        let d = self.inner.delegate_mut();
        if d.blame_state.is_some() {
            d.blame_state = None;
        } else {
            let root = d.root_dir.clone();
            let rel = self.path().strip_prefix(&root).unwrap_or(self.path()).to_path_buf();
            self.inner.delegate_mut().blame_state = Some(blame_async(&root, &rel));
        }
        self.inner.mark_dirty();
    }

    pub fn set_diagnostics(&mut self, diagnostics: Vec<crate::lsp::diagnostics::Diagnostic>) {
        self.inner.delegate_mut().diagnostics = Some(diagnostics);
        self.inner.mark_dirty();
    }

    pub fn clear_diagnostics(&mut self) {
        self.inner.delegate_mut().clear_diagnostics();
    }

    pub fn diagnostic_at_cursor(&self) -> Option<&str> {
        let diags = self.inner.delegate().diagnostics.as_ref()?;
        let line = self.editor().cursor_line();
        diags.iter().find(|d| d.line == line).map(|d| d.message.as_str())
    }

    // --- Diff mode ---
}
