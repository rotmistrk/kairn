//! Accessor methods for KairnDelegate.

use std::path::PathBuf;

use crate::blame::SharedBlame;
use crate::buffer_store::BufferStore;
use crate::lsp::completion::CompletionPopup;
use crate::lsp::diagnostics::Diagnostic;
use crate::settings::EditorSettings;

use super::delegate::KairnDelegate;

impl KairnDelegate {
    pub(crate) fn settings(&self) -> &EditorSettings {
        &self.settings
    }
    pub(crate) fn settings_mut(&mut self) -> &mut EditorSettings {
        &mut self.settings
    }
    pub(crate) fn root_dir(&self) -> &PathBuf {
        &self.root_dir
    }
    pub(crate) fn file_ext(&self) -> &str {
        &self.file_ext
    }
    pub(crate) fn display_title(&self) -> &str {
        &self.display_title
    }
    pub(crate) fn diagnostics_ref(&self) -> &Option<Vec<Diagnostic>> {
        &self.diagnostics
    }
    pub(crate) fn blame_state_ref(&self) -> &Option<SharedBlame> {
        &self.blame_state
    }
    pub(crate) fn highlight_word(&self) -> Option<(usize, usize, usize)> {
        self.highlight_word
    }
    pub(crate) fn set_highlight_word(&mut self, v: Option<(usize, usize, usize)>) {
        self.highlight_word = v;
    }
    pub(crate) fn completion_popup_ref(&self) -> &CompletionPopup {
        &self.completion_popup
    }
    pub(crate) fn diff_state_ref(&self) -> &Option<super::diff_model::DiffState> {
        &self.diff_state
    }
    pub(crate) fn diff_state_mut(&mut self) -> &mut Option<super::diff_model::DiffState> {
        &mut self.diff_state
    }
    pub(crate) fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub(crate) fn set_dirty(&mut self, v: bool) {
        self.dirty = v;
    }
    pub(crate) fn pending_commands_mut(&mut self) -> &mut Vec<(u16, Option<Box<dyn std::any::Any + Send>>)> {
        &mut self.pending_commands
    }
    pub(crate) fn pending_broadcasts_mut(&mut self) -> &mut Vec<(u16, Option<Box<dyn std::any::Any + Send>>)> {
        &mut self.pending_broadcasts
    }
    pub(crate) fn is_save_requested(&self) -> bool {
        self.save_requested
    }
    pub(crate) fn set_save_requested(&mut self, v: bool) {
        self.save_requested = v;
    }
    pub(crate) fn is_force_close(&self) -> bool {
        self.force_close
    }
    pub(crate) fn set_force_close(&mut self, v: bool) {
        self.force_close = v;
    }
    pub(crate) fn pending_diff_ref(&self) -> &Option<String> {
        &self.pending_diff
    }
    pub(crate) fn take_pending_diff(&mut self) -> Option<String> {
        self.pending_diff.take()
    }
    pub(crate) fn is_pending_revert(&self) -> bool {
        self.pending_revert
    }
    pub(crate) fn set_pending_revert(&mut self, v: bool) {
        self.pending_revert = v;
    }
    pub(crate) fn is_pending_nodiff(&self) -> bool {
        self.pending_nodiff
    }
    pub(crate) fn set_pending_nodiff(&mut self, v: bool) {
        self.pending_nodiff = v;
    }
    pub(crate) fn is_eviction_close(&self) -> bool {
        self.eviction_close
    }
    pub(crate) fn set_eviction_close(&mut self, v: bool) {
        self.eviction_close = v;
    }
    pub(crate) fn autosave(&self) -> bool {
        self.settings.autosave
    }
    pub(crate) fn set_disk_mtime(&mut self, v: Option<std::time::SystemTime>) {
        self.disk_mtime = v;
    }
    pub(crate) fn store_mut(&mut self) -> &mut Box<dyn BufferStore> {
        &mut self.store
    }
    pub(crate) fn completion_visible(&self) -> bool {
        self.completion_popup.visible
    }

    /// Scroll viewport so `line` is visible with a 2-line margin.
    /// Use this everywhere instead of duplicating scroll logic.
    pub(crate) fn ensure_line_visible(editor: &mut crate::editor::Editor, line: usize) {
        let h = editor.viewport_height();
        if h == 0 {
            editor.set_viewport_scroll(line);
            return;
        }
        let margin = 2.min(h / 2);
        let scroll = editor.viewport_scroll();
        if line < scroll + margin {
            editor.set_viewport_scroll(line.saturating_sub(margin));
        } else if line + margin >= scroll + h {
            editor.set_viewport_scroll(line + margin + 1 - h);
        }
    }
}
