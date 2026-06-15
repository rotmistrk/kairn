//! NotesView — dedicated view for todo item notes. Wraps EditorView.

use txv_core::prelude::*;

use super::editor::{build as editor_build, EditorView, EditorViewExt};

/// A notes editor, identifiable by type for lookup via `as_any_mut`.
pub struct NotesView {
    pub(crate) editor: EditorView,
}

impl NotesView {
    pub fn new(content: &str) -> Self {
        let mut editor = editor_build::from_text(content);
        editor.set_file_ext("md");
        editor.set_display_title("Notes");
        Self { editor }
    }

    pub fn replace_content(&mut self, content: &str) {
        self.editor.editor_mut().replace_content(content);
        self.editor.mark_dirty();
    }

    pub fn content(&self) -> String {
        self.editor.editor().buf().content()
    }

    pub fn set_store(&mut self, store: Box<dyn crate::buffer_store::BufferStore>) {
        self.editor.set_store(store);
    }
}

impl View for NotesView {
    delegate_view!(editor, override { title, as_any_mut, cursor });

    fn title(&self) -> &str {
        "Notes"
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        self.editor.cursor()
    }
}
