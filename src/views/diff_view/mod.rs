//! DiffView — standalone diff viewer that replaces editor tab during diff mode.
//!
//! Supports unified and side-by-side modes with line numbering and configurable context.

mod draw;
mod handle;

use std::path::PathBuf;

use txv_core::prelude::*;

use crate::views::editor::diff_model::DiffState;

/// Standalone diff view replacing the editor tab.
pub struct DiffView {
    state: ViewState,
    ds: DiffState,
    /// Buffer content lines for rendering Added/Context lines.
    buf_lines: Vec<String>,
    /// Path of the file being diffed.
    path: PathBuf,
    /// Whether to show line numbers.
    show_numbers: bool,
    display_title: String,
    pub(crate) cmd_active: bool,
    pub(crate) cmd_buf: String,
}

impl DiffView {
    pub fn new(ds: DiffState, buf_content: &str, path: PathBuf, show_numbers: bool) -> Self {
        let buf_lines: Vec<String> = buf_content.lines().map(|l| l.to_string()).collect();
        let name = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_default();
        let display_title = format!("[diff] {name}");
        Self {
            state: ViewState::new(ViewOptions::default()),
            ds,
            buf_lines,
            path,
            show_numbers,
            display_title,
            cmd_active: false,
            cmd_buf: String::new(),
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Return the buffer line at the current cursor for jump-back.
    pub fn cursor_buf_line(&self) -> usize {
        self.ds.cursor_buf_line()
    }

    fn height(&self) -> usize {
        self.state.bounds().h() as usize
    }

    fn width(&self) -> u16 {
        self.state.bounds().w()
    }
}

impl View for DiffView {
    fn view_id(&self) -> ViewId {
        ViewId::default()
    }
    fn bounds(&self) -> Rect {
        self.state.bounds()
    }
    fn set_bounds(&mut self, r: Rect) {
        self.state.set_bounds(r);
    }
    fn set_sink(&mut self, sink: txv_core::view::EventSink) {
        self.state.set_sink(sink);
    }
    fn buffer(&self) -> &Buffer {
        self.state.buffer()
    }
    fn needs_redraw(&self) -> bool {
        self.state.is_dirty()
    }
    fn draw(&mut self) {
        self.draw_unified();
    }
    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Key(key) => self.handle_key(key),
            _ => HandleResult::Ignored,
        }
    }
    fn title(&self) -> &str {
        &self.display_title
    }
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
}
