//! EditorView — file viewer wrapping TextArea.

use std::path::{Path, PathBuf};

use txv_core::prelude::*;
use txv_widgets::TextArea;

pub struct EditorView {
    inner: TextArea,
    path: PathBuf,
}

impl EditorView {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let mut area = TextArea::new();
        area.set_content(&content);
        Ok(Self {
            inner: area,
            path: path.to_path_buf(),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl View for EditorView {
    fn bounds(&self) -> Rect { self.inner.bounds() }
    fn set_bounds(&mut self, r: Rect) { self.inner.set_bounds(r); }
    fn options(&self) -> ViewOptions { self.inner.options() }
    fn title(&self) -> &str {
        self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
    }
    fn needs_redraw(&self) -> bool { self.inner.needs_redraw() }
    fn mark_redrawn(&mut self) { self.inner.mark_redrawn(); }
    fn select(&mut self) { self.inner.select(); }
    fn unselect(&mut self) { self.inner.unselect(); }

    fn draw(&self, surface: &mut Surface) {
        self.inner.draw(surface);
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        self.inner.handle(event, queue)
    }
}
