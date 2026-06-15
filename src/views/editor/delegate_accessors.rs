//! Accessor methods and constructor for KairnDelegate.

use std::path::PathBuf;

use crate::blame::SharedBlame;
use crate::buffer_store::BufferStore;
use crate::completer::new_command_list;
use crate::lsp::diagnostics::Diagnostic;
use crate::settings::EditorSettings;

use super::delegate::KairnDelegate;

impl KairnDelegate {
    pub(crate) fn new(settings: EditorSettings, store: Box<dyn BufferStore>) -> Self {
        Self {
            settings,
            root_dir: PathBuf::from("."),
            path: PathBuf::new(),
            file_ext: String::new(),
            display_title: String::new(),
            store,
            disk_mtime: None,
            last_edit_tick: 0,
            current_tick: 0,
            eviction_close: false,
            buffer_id: None,
            view_id: 0,
            diagnostics: None,
            blame_state: None,
            completion_items: Vec::new(),
            completion_visible: false,
            gutter_signs: Vec::new(),
            highlight_word: None,
            diff_state: None,
            command_list: new_command_list(),
            pending_commands: Vec::new(),
            pending_broadcasts: Vec::new(),
            dirty: false,
            save_requested: false,
            force_close: false,
            pending_diff: None,
            pending_revert: false,
            pending_nodiff: false,
            search_hist: None,
            cmd_hist: None,
        }
    }

    pub(crate) fn emit(&mut self, id: u16, data: Option<Box<dyn std::any::Any + Send>>) {
        self.pending_commands.push((id, data));
    }

    pub(crate) fn emit_broadcast(&mut self, id: u16, data: Option<Box<dyn std::any::Any + Send>>) {
        self.pending_broadcasts.push((id, data));
    }

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
    pub(crate) fn set_pending_diff(&mut self, v: String) {
        self.pending_diff = Some(v);
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
        self.completion_visible
    }
    pub(crate) fn set_buffer_id(&mut self, id: Option<crate::buffer_registry::BufferId>) {
        self.buffer_id = id;
    }
    pub(crate) fn buffer_id(&self) -> Option<crate::buffer_registry::BufferId> {
        self.buffer_id
    }
    pub(crate) fn set_store(&mut self, store: Box<dyn BufferStore>) {
        self.store = store;
    }
    pub(crate) fn set_root_dir(&mut self, root: PathBuf) {
        self.root_dir = root;
    }
    pub(crate) fn set_display_title(&mut self, title: String) {
        self.display_title = title;
    }
    pub(crate) fn set_file_ext(&mut self, ext: String) {
        self.file_ext = ext;
    }
    pub(crate) fn set_diagnostics(&mut self, diags: Vec<Diagnostic>) {
        self.diagnostics = Some(diags);
    }
    pub(crate) fn set_gutter_signs(&mut self, signs: Vec<(usize, crate::gutter_signs::GutterSign)>) {
        self.gutter_signs = signs;
    }
    pub(crate) fn set_view_id(&mut self, id: txv_core::view::ViewId) {
        self.view_id = id;
    }
}
