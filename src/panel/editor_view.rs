//! Editor view: renders the Phase A Editor into a txv Surface.

use crossterm::event::KeyEvent;
use txv::cell::Style;
use txv::surface::Surface;
use txv_widgets::{EventResult, ScrollView, Widget, WidgetAction};

use crate::buffer::piece_table::PieceTable;
use crate::editor::command::{Command, EditorAction};
use crate::editor::keymap::Keymap;
use crate::editor::keymap_vim::VimKeymap;
use crate::editor::Editor;

/// Editor viewport: wraps `Editor` and renders with line numbers.
pub struct EditorView {
    editor: Editor,
    keymap: VimKeymap,
    scroll: ScrollView,
    show_line_numbers: bool,
    viewport_h: u16,
}

impl EditorView {
    /// Create a new editor view with an empty buffer.
    pub fn new() -> Self {
        let buf = PieceTable::new();
        // Editor needs a keymap too, but we drive keys externally.
        let editor_keymap = Box::new(VimKeymap::new());
        Self {
            editor: Editor::new(buf, editor_keymap),
            keymap: VimKeymap::new(),
            scroll: ScrollView::new(),
            show_line_numbers: true,
            viewport_h: 24,
        }
    }

    /// Open a file into the editor.
    pub fn open_file(&mut self, path: &str) -> anyhow::Result<()> {
        let buf = PieceTable::from_file(path)?;
        let editor_keymap = Box::new(VimKeymap::new());
        self.editor = Editor::new(buf, editor_keymap);
        self.keymap = VimKeymap::new();
        self.scroll = ScrollView::new();
        Ok(())
    }

    /// Access the underlying editor.
    pub fn editor(&self) -> &Editor {
        &self.editor
    }

    /// Mutable access to the underlying editor.
    pub fn editor_mut(&mut self) -> &mut Editor {
        &mut self.editor
    }

    /// Execute a command on the editor.
    pub fn execute(&mut self, cmd: Command) -> EditorAction {
        let action = self.editor.execute(cmd);
        self.sync_scroll();
        action
    }

    /// Set viewport height.
    pub fn set_viewport_height(&mut self, h: u16) {
        self.viewport_h = h;
        self.editor.set_viewport_height(h as usize);
    }

    /// Width of the line number gutter.
    fn gutter_width(&self) -> u16 {
        if !self.show_line_numbers {
            return 0;
        }
        let lines = self.editor.buffer().line_count().max(1);
        let digits = format!("{lines}").len() as u16;
        digits + 3 // digits + space + "│" + space
    }

    /// Sync scroll state to keep cursor visible.
    fn sync_scroll(&mut self) {
        let (line, _col) = self.editor.cursor();
        let total = self.editor.buffer().line_count();
        self.scroll.set_content_size(total, 0);
        self.scroll.ensure_visible(line, self.viewport_h);
    }
}

impl Widget for EditorView {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        let gw = self.gutter_width();
        let h = surface.height();
        let w = surface.width();
        let (cursor_line, cursor_col) = self.editor.cursor();
        let total_lines = self.editor.buffer().line_count();
        let range = self.scroll.visible_range(h);

        let text_style = Style::default();
        let gutter_style = Style {
            fg: txv::cell::Color::Palette(243),
            ..Style::default()
        };
        let cursor_line_bg = Style {
            bg: txv::cell::Color::Palette(236),
            ..Style::default()
        };

        for (vi, line_idx) in range.enumerate() {
            let row = vi as u16;
            if row >= h {
                break;
            }

            let is_cursor_line = line_idx == cursor_line;

            // Line number gutter.
            if self.show_line_numbers && gw > 0 && line_idx < total_lines {
                let num = format!("{:>width$}", line_idx + 1, width = (gw - 3) as usize,);
                surface.print(0, row, &num, gutter_style);
                surface.print(gw - 2, row, "│", gutter_style);
            }

            // Line content.
            if let Some(line_text) = self.editor.buffer().line(line_idx) {
                let text = line_text.trim_end_matches('\n');
                let style = if is_cursor_line {
                    cursor_line_bg
                } else {
                    text_style
                };
                surface.print(gw, row, text, style);

                // Fill rest of cursor line with highlight.
                if is_cursor_line {
                    let text_len = txv::text::display_width(text) as u16;
                    let start = gw + text_len;
                    if start < w {
                        surface.hline(start, row, w - start, ' ', cursor_line_bg);
                    }
                }
            }
        }

        // Render cursor.
        if _focused {
            // Cursor position relative to viewport.
            let vis_row = cursor_line.saturating_sub(self.scroll.scroll_row) as u16;
            let vis_col = gw + cursor_col as u16;
            if vis_row < h && vis_col < w {
                // Reverse video for cursor.
                let cursor_style = Style {
                    attrs: txv::cell::Attrs {
                        reverse: true,
                        ..Default::default()
                    },
                    ..Style::default()
                };
                let ch = self
                    .editor
                    .buffer()
                    .line(cursor_line)
                    .and_then(|l| l.chars().nth(cursor_col))
                    .unwrap_or(' ');
                surface.put(vis_col, vis_row, ch, cursor_style);
            }
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        let mode = self.editor.mode();
        let cmd = self.keymap.handle_key(key, mode, self.viewport_h);

        if cmd == Command::Noop {
            return EventResult::Ignored;
        }

        let action = self.execute(cmd);
        match action {
            EditorAction::None | EditorAction::CursorMoved | EditorAction::ContentChanged => {
                EventResult::Consumed
            }
            _ => EventResult::Action(WidgetAction::Custom(Box::new(action))),
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}
