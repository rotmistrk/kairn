//! EditorView — View wrapper around the Editor core.

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::app::CursorState;
use crate::commands::{CM_SAVE, CM_TAB_CLOSE};
use crate::editor::keymap::Keymap;
use crate::editor::{Editor, EditorAction};

pub struct EditorView {
    state: ViewState,
    pub editor: Editor,
    path: PathBuf,
    pub cursor_state: CursorState,
}

impl EditorView {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let editor = Editor::open(path)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            cursor_state: std::sync::Arc::new(std::sync::Mutex::new(None)),
        })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn gutter_width(&self) -> u16 {
        let lines = self.editor.buffer.line_count();
        let digits = if lines == 0 { 1 } else { (lines as f64).log10() as u16 + 1 };
        digits + 1
    }
}

impl View for EditorView {
    delegate_view_state!(state, override { title, needs_redraw });

    fn title(&self) -> &str {
        self.path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("untitled")
    }

    fn needs_redraw(&self) -> bool {
        true // always redraw for now (cursor blink, etc.)
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        let gutter_w = self.gutter_width();
        let gutter_style = Style { fg: Color::Ansi(8), ..Style::default() };
        let normal = Style::default();
        let cursor_style = Style {
            attrs: Attrs { reverse: true, ..Attrs::default() },
            ..Style::default()
        };

        let scroll = self.editor.viewport_scroll;
        for row in 0..b.h as usize {
            let line_idx = scroll + row;
            let y = b.y + row as u16;
            surface.hline(b.x, y, b.w, ' ', normal);

            if line_idx >= self.editor.buffer.line_count() {
                surface.print(b.x, y, "~", gutter_style);
                continue;
            }

            // Line number
            let num = format!(
                "{:>width$} ",
                line_idx + 1,
                width = (gutter_w - 1) as usize,
            );
            surface.print(b.x, y, &num, gutter_style);

            // Line content
            let line = self.editor.buffer.line(line_idx).unwrap_or_default();
            let text_x = b.x + gutter_w;
            let avail = b.w.saturating_sub(gutter_w) as usize;
            let visible: String = line.chars().take(avail).collect();
            surface.print(text_x, y, &visible, normal);

            // Cursor
            if line_idx == self.editor.cursor_line && self.state.focused {
                let cx = text_x + self.editor.cursor_col as u16;
                if cx < b.x + b.w {
                    let ch = line.chars().nth(self.editor.cursor_col).unwrap_or(' ');
                    surface.put(cx, y, ch, cursor_style);
                }
            }
        }
    }

    fn handle(
        &mut self,
        event: &Event,
        queue: &mut EventQueue,
    ) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };

        let cmd = self.editor.keymap.handle_key(key, self.editor.mode);
        if cmd == crate::editor::command::Command::Noop {
            return HandleResult::Consumed; // consume to avoid leaking keys
        }

        let action = self.editor.execute(cmd);
        match action {
            EditorAction::SaveRequested => {
                // Save the file
                let content = self.editor.buffer.content();
                if crate::editor::save::save_file(&self.path, &content).is_ok() {
                    self.editor.buffer.mark_saved();
                }
                queue.put_command(CM_SAVE, None);
            }
            EditorAction::CloseRequested => {
                queue.put_command(CM_TAB_CLOSE, None);
            }
            _ => {}
        }

        // Ensure cursor is visible
        self.ensure_cursor_visible();
        self.state.dirty = true;
        // Update shared cursor state for testing
        if let Ok(mut cs) = self.cursor_state.lock() {
            *cs = Some((self.editor.cursor_line, self.editor.cursor_col));
        }
        HandleResult::Consumed
    }
}

impl EditorView {
    fn ensure_cursor_visible(&mut self) {
        let h = self.state.bounds.h as usize;
        if h == 0 {
            return;
        }
        self.editor.viewport_height = h;
        if self.editor.cursor_line < self.editor.viewport_scroll {
            self.editor.viewport_scroll = self.editor.cursor_line;
        } else if self.editor.cursor_line >= self.editor.viewport_scroll + h {
            self.editor.viewport_scroll = self.editor.cursor_line - h + 1;
        }
    }
}
