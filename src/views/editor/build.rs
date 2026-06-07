//! EditorView constructors.

use std::cell::RefCell;
use std::fs::metadata;
use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use super::EditorView;
use crate::buffer_store::FileStore;
use crate::editor::Editor;
use crate::highlight::{self, Highlighter};
use crate::highlight_cache::HighlightCache;
use crate::lsp::completion::CompletionPopup;
use crate::settings::EditorSettings;

impl EditorView {
    pub fn editor(&self) -> &Editor {
        &self.editor
    }
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

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
            hl_cache: RefCell::new(HighlightCache::new("")),
            file_ext: String::new(),
            settings: EditorSettings::default(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title: "[cmd output]".to_string(),
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            sbs_state: None,
            completion_popup: CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: None,
            gutter_signs: Vec::new(),
            store: Box::new(FileStore::new(PathBuf::from("[cmd output]"))),
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
            hl_cache: RefCell::new(HighlightCache::new(&file_ext)),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            sbs_state: None,
            completion_popup: CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: metadata(&path).and_then(|m| m.modified()).ok(),
            gutter_signs: Vec::new(),
            store: Box::new(FileStore::new(path.clone())),
        };
        view.apply_settings();
        view
    }

    fn detect_extension(editor: &Editor, path: &Path) -> String {
        let ext = highlight::extension_from_path(path).to_string();
        if !ext.is_empty() {
            return ext;
        }
        editor
            .buf()
            .line(0)
            .and_then(|line| highlight::extension_from_shebang(&line))
            .unwrap_or_default()
            .to_string()
    }

    fn build(editor: Editor, path: &Path, settings: &EditorSettings) -> Self {
        let file_ext = Self::detect_extension(&editor, path);
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
            hl_cache: RefCell::new(HighlightCache::new(&file_ext)),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            sbs_state: None,
            completion_popup: CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: metadata(path).and_then(|m| m.modified()).ok(),
            gutter_signs: Vec::new(),
            store: Box::new(FileStore::new(path.to_path_buf())),
        }
    }

    fn build_with_theme(editor: Editor, path: &Path, settings: &EditorSettings, syntax_theme: &str) -> Self {
        let file_ext = Self::detect_extension(&editor, path);
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
            hl_cache: RefCell::new(HighlightCache::new(&file_ext)),
            file_ext,
            settings: settings.clone(),
            last_edit_tick: 0,
            tick_counter: 0,
            eviction_close: false,
            display_title,
            diagnostics: None,
            blame_state: None,
            diff_state: None,
            sbs_state: None,
            completion_popup: CompletionPopup::new(),
            buffer_id: None,
            highlight_word: None,
            disk_mtime: metadata(path).and_then(|m| m.modified()).ok(),
            gutter_signs: Vec::new(),
            store: Box::new(FileStore::new(path.to_path_buf())),
        }
    }

    pub fn set_root_dir(&mut self, root: PathBuf) {
        self.root_dir = root;
        self.refresh_gutter_signs();
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editor_settings::EditorSettings;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn shebang_bash_sets_sh_extension() {
        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "#!/bin/bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        f.flush().unwrap();

        // Rename to extensionless path
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("my-script");
        std::fs::copy(f.path(), &target).unwrap();

        let settings = EditorSettings::default();
        let view = EditorView::open(&target, &settings).unwrap();
        assert_eq!(view.file_ext, "sh", "shebang #!/bin/bash should detect as sh");
    }

    #[test]
    fn syntect_finds_sh_syntax() {
        use syntect::parsing::SyntaxSet;
        let ss = SyntaxSet::load_defaults_newlines();
        let by_ext = ss.find_syntax_by_extension("sh");
        assert!(by_ext.is_some(), "syntect should find 'sh' by extension");
    }

    #[test]
    fn shebang_file_produces_highlighted_spans() {
        use txv_core::prelude::*;

        let mut f = NamedTempFile::new().unwrap();
        writeln!(f, "#!/bin/bash").unwrap();
        writeln!(f, "echo hello").unwrap();
        writeln!(f, "if [ -f /tmp/x ]; then").unwrap();
        writeln!(f, "  rm /tmp/x").unwrap();
        writeln!(f, "fi").unwrap();
        f.flush().unwrap();

        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("pre-commit");
        std::fs::copy(f.path(), &target).unwrap();

        let settings = EditorSettings::default();
        let mut view = EditorView::open(&target, &settings).unwrap();
        view.set_bounds(Rect::new(0, 0, 80, 24));
        view.draw();

        // Check that the highlight cache produced non-trivial spans
        let spans = view.compute_viewport_spans(0, 5);
        assert!(!spans.is_empty(), "should have spans");
        // At least one span should have non-default fg (colored)
        let has_color = spans.iter().flatten().any(|s| s.style().fg() != Style::default().fg());
        assert!(
            has_color,
            "shebang file should have syntax coloring, ext={}",
            view.file_ext
        );
    }
}
