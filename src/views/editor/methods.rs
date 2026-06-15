//! EditorView utility methods via extension trait.

use std::fs::metadata;
use std::path::PathBuf;

use txv_edit::view::draw::compute_gutter_width;

use super::EditorView;
use crate::blame::blame_async;
use crate::buffer_registry::BufferId;
use crate::gutter_signs::compute_gutter_signs;

pub trait EditorViewExt {
    fn buffer_id(&self) -> Option<BufferId>;
    fn set_buffer_id(&mut self, id: Option<BufferId>);
    fn refresh_gutter_signs(&mut self);
    fn save_now(&mut self) -> bool;
    fn save(&mut self) -> Result<(), String>;
    fn language(&self) -> &str;
    fn apply_settings(&mut self);
    fn set_syntax_theme(&mut self, name: &str);
    fn invalidate_highlight(&mut self);
    fn set_store(&mut self, store: Box<dyn crate::buffer_store::BufferStore>);
    fn set_root_dir(&mut self, root: PathBuf);
    fn request_close(&mut self);
    fn goto(&mut self, line: u32, col: u32);
    fn scroll_to_line(&mut self, line: usize);
    fn undo(&mut self);
    fn redo(&mut self);
    fn sync_title(&mut self);
    fn set_file_ext(&mut self, ext: &str);
    fn set_display_title(&mut self, title: &str);
    fn gutter_width(&self) -> u16;
    fn toggle_blame(&mut self);
    fn set_diagnostics(&mut self, diagnostics: Vec<crate::lsp::diagnostics::Diagnostic>);
    fn clear_diagnostics(&mut self);
    fn diagnostic_at_cursor(&self) -> Option<&str>;
    fn set_gutter_signs_data(&mut self, signs: Vec<(usize, crate::gutter_signs::GutterSign)>);
}

impl EditorViewExt for EditorView {
    fn buffer_id(&self) -> Option<BufferId> {
        self.delegate().buffer_id()
    }

    fn set_buffer_id(&mut self, id: Option<BufferId>) {
        self.delegate_mut().set_buffer_id(id);
    }

    fn refresh_gutter_signs(&mut self) {
        let d = self.delegate();
        if !d.settings().gutter_signs() {
            let dm = self.delegate_mut();
            dm.gutter_signs.clear();
            return;
        }
        let root = d.root_dir().clone();
        let rel = self
            .path()
            .strip_prefix(&root)
            .unwrap_or(self.path())
            .to_string_lossy()
            .to_string();
        let content = self.editor().buf().content();
        let signs = compute_gutter_signs(&root, &rel, &content);
        let dm = self.delegate_mut();
        dm.gutter_signs = signs;
    }

    fn save_now(&mut self) -> bool {
        self.save_buffer_impl()
    }

    fn save(&mut self) -> Result<(), String> {
        let content = self.editor().buf().content();
        self.delegate_mut().store_mut().save(&content)?;
        self.editor_mut().buf().mark_saved();
        Ok(())
    }

    fn language(&self) -> &str {
        self.delegate().file_ext()
    }

    fn apply_settings(&mut self) {
        let s = self.delegate().settings().clone();
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

    fn set_syntax_theme(&mut self, name: &str) {
        self.highlighter_mut().set_theme(name);
        self.hl_cache_mut().invalidate_all();
        self.mark_dirty();
    }

    fn invalidate_highlight(&mut self) {
        self.hl_cache_mut().invalidate_all();
        self.mark_dirty();
    }

    fn set_store(&mut self, store: Box<dyn crate::buffer_store::BufferStore>) {
        self.delegate_mut().set_store(store);
    }

    fn set_root_dir(&mut self, root: PathBuf) {
        self.delegate_mut().set_root_dir(root);
        self.refresh_gutter_signs();
    }

    fn request_close(&mut self) {
        if !self.delegate().autosave() {
            self.delegate_mut().set_eviction_close(true);
        }
        self.mark_dirty();
    }

    fn goto(&mut self, line: u32, col: u32) {
        use crate::editor::ephemeral::HighlightOwner;
        use crate::editor::ephemeral_range::EphemeralRange;
        let max_line = self.editor().buf().line_count().saturating_sub(1);
        let target_line = (line as usize).min(max_line);
        self.editor_mut().set_cursor_line(target_line);
        self.editor_mut().set_cursor_col(col as usize);
        self.editor_mut()
            .ephemeral_mut()
            .set(vec![EphemeralRange::full_line(target_line)], HighlightOwner::Transient);
        self.scroll_to_line(target_line);
        self.mark_dirty();
    }

    fn scroll_to_line(&mut self, _line: usize) {
        self.ensure_cursor_visible();
    }

    fn undo(&mut self) {
        self.editor_mut().buf().undo();
        self.editor_mut().clamp_cursor();
        self.mark_dirty();
    }

    fn redo(&mut self) {
        self.editor_mut().buf().redo();
        self.editor_mut().clamp_cursor();
        self.mark_dirty();
    }

    fn sync_title(&mut self) {
        let name = self
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        self.delegate_mut().set_display_title(name);
    }

    fn set_file_ext(&mut self, ext: &str) {
        self.delegate_mut().set_file_ext(ext.to_string());
    }

    fn set_display_title(&mut self, title: &str) {
        self.delegate_mut().set_display_title(title.to_string());
    }

    fn gutter_width(&self) -> u16 {
        compute_gutter_width(self.editor(), self.delegate())
    }

    fn toggle_blame(&mut self) {
        let d = self.delegate_mut();
        if d.blame_state.is_some() {
            d.blame_state = None;
        } else {
            let root = d.root_dir.clone();
            let rel = d.path.strip_prefix(&root).unwrap_or(&d.path).to_path_buf();
            d.blame_state = Some(blame_async(&root, &rel));
        }
        self.mark_dirty();
    }

    fn set_diagnostics(&mut self, diagnostics: Vec<crate::lsp::diagnostics::Diagnostic>) {
        self.delegate_mut().set_diagnostics(diagnostics);
        self.mark_dirty();
    }

    fn clear_diagnostics(&mut self) {
        self.delegate_mut().clear_diagnostics();
    }

    fn diagnostic_at_cursor(&self) -> Option<&str> {
        let diags = self.delegate().diagnostics_ref().as_ref()?;
        let line = self.editor().cursor_line();
        diags.iter().find(|d| d.line == line).map(|d| d.message.as_str())
    }

    fn set_gutter_signs_data(&mut self, signs: Vec<(usize, crate::gutter_signs::GutterSign)>) {
        self.delegate_mut().set_gutter_signs(signs);
        self.mark_dirty();
    }
}

// Private helper
trait EditorViewPrivate {
    fn save_buffer_impl(&mut self) -> bool;
}

impl EditorViewPrivate for EditorView {
    fn save_buffer_impl(&mut self) -> bool {
        let content = self.editor().buf().content();
        if self.delegate_mut().store_mut().save(&content).is_ok() {
            self.editor_mut().buf().mark_saved();
            let mtime = metadata(self.path()).and_then(|m| m.modified()).ok();
            self.delegate_mut().set_disk_mtime(mtime);
            self.refresh_gutter_signs();
            true
        } else {
            false
        }
    }
}
