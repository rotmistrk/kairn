//! EditorView constructors.

use std::fs::{metadata, read_to_string};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use txv_core::prelude::*;
use txv_core::view::EventSink;
use txv_edit::view::EditorView as TxvEditorView;

use super::delegate::KairnDelegate;
use super::EditorView;
use crate::buffer::piece_table::PieceTable;
use crate::buffer_store::FileStore;
use crate::editor::Editor;
use crate::highlight::{self};
use crate::settings::EditorSettings;

impl EditorView {
    pub fn editor(&self) -> &Editor {
        self.inner.editor()
    }

    pub fn editor_mut(&mut self) -> &mut Editor {
        self.inner.editor_mut()
    }

    pub fn path(&self) -> &Path {
        self.inner.path()
    }

    pub fn open(path: &Path, settings: &EditorSettings) -> anyhow::Result<Self> {
        let mut view = Self::build(path, settings, None)?;
        view.apply_settings();
        Ok(view)
    }

    pub fn open_with_theme(path: &Path, settings: &EditorSettings, syntax_theme: &str) -> anyhow::Result<Self> {
        let mut view = Self::build(path, settings, Some(syntax_theme))?;
        view.apply_settings();
        Ok(view)
    }

    pub fn new_file(path: &Path, settings: &EditorSettings) -> Self {
        let delegate = make_delegate(path, settings);
        let mut inner = TxvEditorView::with_delegate(delegate);
        inner.set_path(path);
        Self { inner, buffer_id: None }
    }

    pub fn from_text(content: &str) -> Self {
        let settings = EditorSettings::default();
        let delegate = KairnDelegate::new(settings, Box::new(FileStore::new(PathBuf::from("[cmd output]"))));
        let mut d = delegate;
        d.display_title = "[cmd output]".to_string();
        let mut inner = TxvEditorView::with_delegate(d);
        inner.set_content(content, "");
        let sink = EventSink::new();
        inner.set_sink(sink);
        Self { inner, buffer_id: None }
    }

    pub fn from_arc_buffer(
        buf: Arc<Mutex<PieceTable>>,
        file_path: Option<String>,
        settings: &EditorSettings,
        syntax_theme: &str,
    ) -> Self {
        let path = file_path.as_deref().map(PathBuf::from).unwrap_or_default();
        let file_ext = highlight::extension_from_path(&path).to_string();
        let display_title = path.file_name().and_then(|n| n.to_str()).unwrap_or("split").to_string();
        let mut delegate = KairnDelegate::new(settings.clone(), Box::new(FileStore::new(path.clone())));
        delegate.file_ext = file_ext;
        delegate.display_title = display_title;
        delegate.disk_mtime = metadata(&path).and_then(|m| m.modified()).ok();
        let editor = Editor::with_arc(buf);
        let mut inner = TxvEditorView::with_delegate(delegate);
        *inner.editor_mut() = editor;
        inner.set_path(&path);
        inner.highlighter_mut().set_theme(syntax_theme);
        let mut view = Self { inner, buffer_id: None };
        view.apply_settings();
        view
    }

    fn build(path: &Path, settings: &EditorSettings, theme: Option<&str>) -> anyhow::Result<Self> {
        let editor = Editor::open(path).map_err(|e| anyhow::anyhow!("{}", e))?;
        let delegate = make_delegate(path, settings);
        let mut inner = TxvEditorView::with_delegate(delegate);
        *inner.editor_mut() = editor;
        inner.set_path(path);
        if let Some(t) = theme {
            inner.highlighter_mut().set_theme(t);
        }
        Ok(Self { inner, buffer_id: None })
    }
}

fn make_delegate(path: &Path, settings: &EditorSettings) -> KairnDelegate {
    let file_ext = detect_extension(path);
    let display_title = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled")
        .to_string();
    let store = Box::new(FileStore::new(path.to_path_buf()));
    let mut d = KairnDelegate::new(settings.clone(), store);
    d.file_ext = file_ext;
    d.display_title = display_title;
    d.disk_mtime = metadata(path).and_then(|m| m.modified()).ok();
    d.path = path.to_path_buf();
    d
}

fn detect_extension(path: &Path) -> String {
    let ext = highlight::extension_from_path(path).to_string();
    if !ext.is_empty() {
        return ext;
    }
    // Try shebang detection from first line
    if let Ok(content) = read_to_string(path) {
        if let Some(first_line) = content.lines().next() {
            if let Some(ext) = highlight::extension_from_shebang(first_line) {
                return ext.to_string();
            }
        }
    }
    String::new()
}
