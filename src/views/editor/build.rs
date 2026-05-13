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
}
