//! EditorView constructors.

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use super::EditorView;
use crate::editor::Editor;
use crate::highlight::{self, Highlighter};
use crate::settings::EditorSettings;

impl EditorView {
    pub fn open(path: &Path, settings: &EditorSettings) -> anyhow::Result<Self> {
        let editor = Editor::open(path).map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut view = Self::build(editor, path, settings);
        view.apply_settings();
        Ok(view)
    }

    pub fn open_with_theme(path: &Path, settings: &EditorSettings, syntax_theme: &str) -> anyhow::Result<Self> {
        let editor = Editor::open(path).map_err(|e| anyhow::anyhow!("{}", e))?;
        let mut view = Self::build_with_theme(editor, path, settings, syntax_theme);
        view.apply_settings();
        Ok(view)
    }

    pub fn new_file(path: &Path, settings: &EditorSettings) -> Self {
        let mut view = Self::build(Editor::from_text(""), path, settings);
        view.apply_settings();
        view
    }

    pub fn from_text(content: &str) -> Self {
        Self {
            state: ViewState::default(),
            editor: Editor::from_text(content),
            path: PathBuf::from("[cmd output]"),
            root_dir: PathBuf::from("."),
            highlighter: Highlighter::new(),
            hl_cache: std::cell::RefCell::new(crate::highlight_cache::HighlightCache::new("")),
            file_ext: String::new(),
            settings: EditorSettings::default(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title: "[cmd output]".to_string(),
            diagnostics: None,
            diag_suppressed: false,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: None,
            store: Box::new(crate::buffer_store::FileStore::new(PathBuf::from("[cmd output]"))),
        }
    }

    pub fn from_arc_buffer(
        buf: std::sync::Arc<std::sync::Mutex<crate::buffer::piece_table::PieceTable>>,
        file_path: Option<String>,
        settings: &EditorSettings,
        syntax_theme: &str,
    ) -> Self {
        let path = file_path.as_deref().map(PathBuf::from).unwrap_or_default();
        let file_ext = highlight::extension_from_path(&path).to_string();
        let display_title = path.file_name().and_then(|n| n.to_str()).unwrap_or("split").to_string();
        let mut view = Self {
            state: ViewState::default(),
            editor: Editor::with_arc(buf),
            path: path.clone(),
            root_dir: path.parent().unwrap_or(Path::new(".")).to_path_buf(),
            highlighter: Highlighter::with_theme(syntax_theme),
            hl_cache: std::cell::RefCell::new(crate::highlight_cache::HighlightCache::new(&file_ext)),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            diag_suppressed: false,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: std::fs::metadata(&path).and_then(|m| m.modified()).ok(),
            store: Box::new(crate::buffer_store::FileStore::new(path.clone())),
        };
        view.apply_settings();
        view
    }

    fn build(editor: Editor, path: &Path, settings: &EditorSettings) -> Self {
        let file_ext = highlight::extension_from_path(path).to_string();
        let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let display_title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            root_dir,
            highlighter: Highlighter::new(),
            hl_cache: std::cell::RefCell::new(crate::highlight_cache::HighlightCache::new(&file_ext)),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            diag_suppressed: false,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: std::fs::metadata(path).and_then(|m| m.modified()).ok(),
            store: Box::new(crate::buffer_store::FileStore::new(path.to_path_buf())),
        }
    }

    fn build_with_theme(editor: Editor, path: &Path, settings: &EditorSettings, syntax_theme: &str) -> Self {
        let file_ext = highlight::extension_from_path(path).to_string();
        let root_dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
        let display_title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            root_dir,
            highlighter: Highlighter::with_theme(syntax_theme),
            hl_cache: std::cell::RefCell::new(crate::highlight_cache::HighlightCache::new(&file_ext)),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            diag_suppressed: false,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: std::fs::metadata(path).and_then(|m| m.modified()).ok(),
            store: Box::new(crate::buffer_store::FileStore::new(path.to_path_buf())),
        }
    }

    pub fn set_root_dir(&mut self, root: PathBuf) {
        self.root_dir = root;
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn language(&self) -> &str {
        &self.file_ext
    }

    pub(super) fn apply_settings(&mut self) {
        self.editor.options.wrap = self.settings.wrap;
        self.editor.options.list = self.settings.list;
        self.editor.options.tab_width = self.settings.tabstop as usize;
        self.editor.options.number = self.settings.number;
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
        let max_line = self.editor.buf().line_count().saturating_sub(1);
        self.editor.cursor_line = (line as usize).min(max_line);
        self.editor.cursor_col = col as usize;
        self.ensure_cursor_visible();
        if self.state.bounds().h == 0 {
            self.editor.viewport_scroll = self.editor.cursor_line;
        }
        self.state.mark_dirty();
    }

    pub(super) fn gutter_width(&self) -> u16 {
        if !self.editor.options.number {
            return 0;
        }
        let lines = self.editor.buf().line_count();
        let digits = if lines == 0 {
            1
        } else {
            (lines as f64).log10() as u16 + 1
        };
        let blame_w = if self.blame_state.is_some() {
            24
        } else {
            0
        };
        digits + 1 + blame_w
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
            self.blame_state = Some(crate::blame::blame_async(&self.root_dir, &rel));
        }
        self.state.mark_dirty();
    }
}
