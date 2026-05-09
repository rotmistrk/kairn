//! EditorView — wraps the Editor (piece table + keymap) as a View.
//!
//! Translates key events through the keymap into Commands, executes
//! them on the Editor, and renders the buffer with line numbers.

use std::path::Path;

use txv::cell::{Color, Style};
use txv::layout::Rect;
use txv::surface::Surface;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};

use crate::buffer::PieceTable;
use crate::commands::{CM_CLOSE, CM_SAVE};
use crate::editor::command::EditorAction;
use crate::editor::{Editor, KeymapKind};
use crate::types::CommandOutbox;

/// Editor view: buffer + keymap + rendering.
pub struct EditorView {
    editor: Editor,
    title: String,
    bounds: Rect,
    pub outbox: CommandOutbox,
}

impl EditorView {
    /// Open a file into an editor view.
    pub fn open(path: &Path, keymap: KeymapKind) -> Option<Self> {
        let path_str = path.to_str()?;
        let buffer = PieceTable::from_file(path_str).ok()?;
        let title = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
            .to_string();
        let km = keymap.create_keymap();
        Some(Self {
            editor: Editor::new(buffer, km),
            title,
            bounds: Rect { x: 0, y: 0, w: 0, h: 0 },
            outbox: CommandOutbox::default(),
        })
    }

    /// The tab title for this editor.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Line number gutter width.
    fn gutter_width(&self) -> u16 {
        let lines = self.editor.buffer().line_count();
        let digits = format!("{}", lines).len() as u16;
        digits.max(3) + 1 // at least 3 digits + 1 space
    }

    /// Draw line numbers in the gutter.
    fn draw_gutter(&self, surface: &mut Surface<'_>, scroll: usize) {
        let h = surface.height() as usize;
        let gw = self.gutter_width();
        let style = Style {
            fg: Color::Ansi(8), // dim
            ..Style::default()
        };
        for row in 0..h {
            let line_num = scroll + row + 1;
            if line_num > self.editor.buffer().line_count() {
                break;
            }
            let num_str = format!("{:>width$} ", line_num, width = (gw - 1) as usize);
            surface.print(0, row as u16, &num_str, style);
        }
    }

    /// Draw buffer content.
    fn draw_content(&self, surface: &mut Surface<'_>, scroll: usize) {
        let h = surface.height() as usize;
        let w = surface.width() as usize;
        let style = Style::default();

        for row in 0..h {
            let line_idx = scroll + row;
            if let Some(line) = self.editor.buffer().line(line_idx) {
                let display: String = line.chars().take(w).collect();
                surface.print(0, row as u16, &display, style);
            }
        }
    }

    /// Draw cursor highlight.
    fn draw_cursor(&self, surface: &mut Surface<'_>, scroll: usize, gutter: u16) {
        let (cl, cc) = self.editor.cursor();
        if cl < scroll {
            return;
        }
        let row = (cl - scroll) as u16;
        if row >= surface.height() {
            return;
        }
        let col = cc as u16 + gutter;
        if col < surface.width() {
            let cursor_style = Style {
                attrs: txv::cell::Attrs { reverse: true, ..Default::default() },
                ..Style::default()
            };
            // Read current char at cursor position or use space
            surface.put(col, row, '▋', cursor_style);
        }
    }

    fn viewport_scroll(&self) -> usize {
        // Simple: keep cursor in view
        let h = self.bounds.h.saturating_sub(0) as usize;
        let (cl, _) = self.editor.cursor();
        if h == 0 {
            return 0;
        }
        // This is a simplified scroll — real impl would track scroll state
        cl.saturating_sub(h / 2)
    }
}

impl View for EditorView {
    fn draw(&self, surface: &mut Surface<'_>, _ctx: &DrawContext) {
        let gw = self.gutter_width();
        let w = surface.width();
        let h = surface.height();
        if w == 0 || h == 0 {
            return;
        }
        let scroll = self.viewport_scroll();

        // Draw gutter
        let mut gutter_sub = surface.sub(0, 0, gw, h);
        self.draw_gutter(&mut gutter_sub, scroll);

        // Draw content area
        let content_w = w.saturating_sub(gw);
        if content_w > 0 {
            let mut content_sub = surface.sub(gw, 0, content_w, h);
            self.draw_content(&mut content_sub, scroll);
        }

        // Draw cursor
        self.draw_cursor(surface, scroll, gw);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let key = match event {
            Event::Key(k) => *k,
            _ => return HandleResult::Ignored,
        };

        // Update viewport height for page calculations
        self.editor.set_viewport_height(self.bounds.h as usize);

        // Translate key through keymap and execute
        let action = self.editor.handle_key(key);

        // Translate EditorAction to outbox commands
        match action {
            EditorAction::SaveRequested => self.outbox.emit(CM_SAVE),
            EditorAction::CloseRequested | EditorAction::ForceCloseRequested => {
                self.outbox.emit(CM_CLOSE);
            }
            _ => {}
        }

        HandleResult::Consumed
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
    }
}
