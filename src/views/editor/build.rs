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
            file_ext: String::new(),
            settings: EditorSettings::default(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title: "[cmd output]".to_string(),
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
        }
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
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
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
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            completion_popup: crate::lsp::completion::CompletionPopup::new(),
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
    }

    pub fn save(&mut self) -> Result<(), String> {
        let content = self.editor.buffer.content();
        crate::editor::save::save_file(&self.path, &content).map_err(|e| e.to_string())?;
        self.editor.buffer.mark_saved();
        Ok(())
    }

    pub fn request_close(&mut self) {
        if self.editor.buffer.is_dirty() && !self.settings.autosave {
            self.eviction_close = true;
            self.state.mark_dirty();
        }
    }

    pub fn goto(&mut self, line: u32, col: u32) {
        let max_line = self.editor.buffer.line_count().saturating_sub(1);
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
        let lines = self.editor.buffer.line_count();
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
