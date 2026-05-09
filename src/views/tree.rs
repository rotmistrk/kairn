//! FileTreeView — wraps txv_widgets::TreeView<FileTreeData>.
//!
//! Translates Enter on a file node into CM_OPEN_FILE command.
//! Delegates all drawing and navigation to the inner TreeView.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crossterm::event::{KeyCode, KeyEvent};
use txv::layout::Rect;
use txv::surface::Surface;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};
use txv_widgets::{FileTreeData, TreeView};

use crate::commands::CM_OPEN_FILE;
use crate::types::{CommandOutbox, OpenFilePayload};

/// File tree panel wrapping the generic TreeView widget.
pub struct FileTreeView {
    inner: TreeView<FileTreeData>,
    outbox: Arc<Mutex<CommandOutbox>>,
}

impl FileTreeView {
    /// Create a file tree rooted at `path`.
    /// The shared outbox allows the App to drain commands.
    pub fn open(path: &Path, outbox: Arc<Mutex<CommandOutbox>>) -> Option<Self> {
        let data = FileTreeData::new(path, 20).ok()?;
        Some(Self {
            inner: TreeView::new(data),
            outbox,
        })
    }

    fn selected_path(&self) -> Option<&PathBuf> {
        self.inner.selected_node()
    }

    fn is_selected_dir(&self) -> bool {
        self.selected_path()
            .map(|p| self.inner.data().is_dir(p))
            .unwrap_or(false)
    }

    fn emit_open_file(&self, path: &Path) {
        if let Ok(mut outbox) = self.outbox.lock() {
            outbox.emit_with(
                CM_OPEN_FILE,
                OpenFilePayload { path: path.to_string_lossy().to_string() },
            );
        }
    }
}

impl View for FileTreeView {
    fn draw(&self, surface: &mut Surface<'_>, ctx: &DrawContext) {
        self.inner.draw(surface, ctx);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        // Intercept Enter on a file to emit CM_OPEN_FILE
        if let Event::Key(KeyEvent { code: KeyCode::Enter, .. }) = event {
            if !self.is_selected_dir() {
                if let Some(path) = self.selected_path().cloned() {
                    self.emit_open_file(&path);
                    return HandleResult::Consumed;
                }
            }
        }

        // Right arrow on a file also opens it
        if let Event::Key(KeyEvent { code: KeyCode::Right, .. }) = event {
            if !self.is_selected_dir() {
                if let Some(path) = self.selected_path().cloned() {
                    self.emit_open_file(&path);
                    return HandleResult::Consumed;
                }
            }
        }

        self.inner.handle(event)
    }

    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.inner.set_bounds(rect);
    }
}
