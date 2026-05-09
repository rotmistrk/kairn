//! EditorView — View wrapper around the Editor core.

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::{CM_SAVE, CM_TAB_CLOSE};
use crate::editor::keymap::Keymap;
use crate::editor::{Editor, EditorAction};
use crate::highlight::{self, Highlighter};

pub struct EditorView {
    state: ViewState,
    pub editor: Editor,
    path: PathBuf,
    highlighter: Highlighter,
    file_ext: String,
}

impl EditorView {
    pub fn open(path: &Path) -> anyhow::Result<Self> {
        let editor = Editor::open(path)
            .map_err(|e| anyhow::anyhow!("{}", e))?;
        let file_ext = highlight::extension_from_path(path).to_string();
        Ok(Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            highlighter: Highlighter::new(),
            file_ext,
        })
    }

    /// Create an editor for a new (non-existent) file with empty buffer.
    pub fn new_file(path: &Path) -> Self {
        let editor = Editor::from_text("");
        let file_ext = highlight::extension_from_path(path).to_string();
        Self {
            state: ViewState::default(),
            editor,
            path: path.to_path_buf(),
            highlighter: Highlighter::new(),
            file_ext,
        }
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
        true
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
        let visual_style = Style {
            attrs: Attrs { reverse: true, ..Attrs::default() },
            fg: Color::Ansi(3),
            ..Style::default()
        };

        let scroll = self.editor.viewport_scroll;

        // Determine visual selection range (byte offsets)
        let visual_range = self.editor.visual_range();

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

            // Line content with syntax highlighting
            let line = self.editor.buffer.line(line_idx).unwrap_or_default();
            let text_x = b.x + gutter_w;
            let avail = b.w.saturating_sub(gutter_w) as usize;

            // Get line start offset for visual selection calculation
            let line_start_off = self.editor.buffer.line_col_to_offset(line_idx, 0).unwrap_or(0);

            // Use highlighter for syntax coloring
            let spans = self.highlighter.highlight_line(&line, &self.file_ext);

            let mut col_offset: u16 = 0;
            let mut byte_pos = line_start_off;
            for span in &spans {
                for ch in span.text.chars() {
                    if col_offset as usize >= avail {
                        break;
                    }
                    let x = text_x + col_offset;
                    let char_col = col_offset as usize;

                    // Determine style: cursor > visual > syntax
                    let style = if line_idx == self.editor.cursor_line
                        && char_col == self.editor.cursor_col
                        && self.state.focused
                    {
                        cursor_style
                    } else if let Some((vs, ve)) = visual_range {
                        if byte_pos >= vs && byte_pos < ve {
                            visual_style
                        } else {
                            span.style
                        }
                    } else {
                        span.style
                    };

                    surface.put(x, y, ch, style);
                    col_offset += 1;
                    byte_pos += ch.len_utf8();
                }
            }

            // If line is empty or shorter, still show cursor
            if line_idx == self.editor.cursor_line
                && self.state.focused
                && self.editor.cursor_col as u16 >= col_offset
            {
                let cx = text_x + self.editor.cursor_col as u16;
                if cx < b.x + b.w {
                    surface.put(cx, y, ' ', cursor_style);
                }
            }
        }

        // Render command/search prompt at bottom of editor area
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let prompt_y = b.y + b.h.saturating_sub(1);
            let prompt_style = Style {
                attrs: Attrs { reverse: true, ..Attrs::default() },
                ..Style::default()
            };
            surface.hline(b.x, prompt_y, b.w, ' ', prompt_style);
            let prefix = if self.editor.mode == crate::editor::keymap::EditorMode::Search { "/" } else { ":" };
            surface.print(b.x, prompt_y, prefix, prompt_style);
            surface.print(b.x + 1, prompt_y, &self.editor.command_buf, prompt_style);
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

        // In command/search mode, handle input directly
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            return self.handle_command_input(key, queue);
        }

        let cmd = self.editor.keymap.handle_key(key, self.editor.mode);
        if cmd == crate::editor::command::Command::Noop {
            return HandleResult::Consumed;
        }

        let action = self.editor.execute(cmd);
        self.handle_action(action, queue);
        self.ensure_cursor_visible();
        self.state.dirty = true;
        HandleResult::Consumed
    }
}

impl EditorView {
    fn handle_command_input(
        &mut self,
        key: &txv_core::event::KeyEvent,
        queue: &mut EventQueue,
    ) -> HandleResult {
        use txv_core::event::KeyCode;

        match &key.code {
            KeyCode::Esc => {
                self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                self.editor.command_buf.clear();
            }
            KeyCode::Enter => {
                let buf = self.editor.command_buf.clone();
                if self.editor.mode == crate::editor::keymap::EditorMode::Search {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                    let action = self.editor.execute(
                        crate::editor::command::Command::SearchForward(buf),
                    );
                    self.handle_action(action, queue);
                } else {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                    let action = self.editor.execute(
                        crate::editor::command::Command::ExCommand(buf),
                    );
                    self.handle_action(action, queue);
                }
                self.editor.command_buf.clear();
            }
            KeyCode::Backspace => {
                if self.editor.command_buf.is_empty() {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                } else {
                    self.editor.command_buf.pop();
                }
            }
            KeyCode::Char(c) => {
                self.editor.command_buf.push(*c);
            }
            _ => {}
        }

        self.ensure_cursor_visible();
        self.state.dirty = true;
        HandleResult::Consumed
    }

    fn handle_action(&mut self, action: EditorAction, queue: &mut EventQueue) {
        match action {
            EditorAction::SaveRequested => {
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
    }

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
