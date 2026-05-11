//! EditorView — View wrapper around the Editor core.

mod build;
mod diff;
mod draw;
mod draw_diagnostics;
mod handle;

use std::path::PathBuf;

use txv_core::prelude::*;

use crate::commands::CM_CLIPBOARD_PASTE;
use crate::commands::CM_TAB_CLOSE;
use crate::editor::keymap::Keymap;
use crate::editor::Editor;
use crate::highlight::Highlighter;
use crate::settings::EditorSettings;

/// Per-line diff tag for inline diff rendering.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum DiffTag {
    Context,
    Added,
    Removed,
}

pub struct EditorView {
    state: ViewState,
    pub editor: Editor,
    path: PathBuf,
    root_dir: PathBuf,
    highlighter: Highlighter,
    file_ext: String,
    pub settings: EditorSettings,
    last_edit_tick: u64,
    tick_counter: u64,
    close_prompt: bool,
    display_title: String,
    diagnostics: Option<Vec<crate::lsp::diagnostics::Diagnostic>>,
    /// Diff mode: per-line tag ('+' added, '-' removed, ' ' context). None = normal mode.
    diff_lines: Option<Vec<DiffTag>>,
    diff_base: String,
}

impl EditorView {
    pub(super) fn apply_settings(&mut self) {
        self.editor.options.wrap = self.settings.wrap;
        self.editor.options.list = self.settings.list;
        self.editor.options.tab_width = self.settings.tabstop as usize;
        self.editor.options.number = self.settings.number;
    }

    fn gutter_width(&self) -> u16 {
        if !self.editor.options.number {
            return 0;
        }
        let lines = self.editor.buffer.line_count();
        let digits = if lines == 0 {
            1
        } else {
            (lines as f64).log10() as u16 + 1
        };
        digits + 1
    }
}

impl View for EditorView {
    delegate_view_state!(state, override { title, needs_redraw });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn needs_redraw(&self) -> bool {
        true
    }

    fn draw(&self, surface: &mut Surface) {
        self.draw_editor(surface);
        self.draw_diagnostics(surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        // Tick: autosave check + completion trigger
        if let Event::Tick = event {
            self.handle_tick(queue);
            return HandleResult::Ignored;
        }

        let Event::Key(key) = event else {
            // Handle paste (bracketed paste from terminal)
            if let Event::Paste(text) = event {
                let offset = self
                    .editor
                    .buffer
                    .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                    .unwrap_or(0);
                self.editor.buffer.insert(offset, text);
                self.last_edit_tick = self.tick_counter;
                self.state.dirty = true;
                return HandleResult::Consumed;
            }
            // Handle clipboard paste command
            if let Event::Command { id, data } = event {
                if *id == crate::commands::CM_DIFF {
                    let args = data
                        .as_ref()
                        .and_then(|b| b.downcast_ref::<String>())
                        .map(|s| s.as_str())
                        .unwrap_or("");
                    self.toggle_diff(args);
                    return HandleResult::Consumed;
                }
                if *id == CM_CLIPBOARD_PASTE {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(text) = boxed.downcast_ref::<String>() {
                            let offset = self
                                .editor
                                .buffer
                                .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
                                .unwrap_or(0);
                            self.editor.buffer.insert(offset, text);
                            self.last_edit_tick = self.tick_counter;
                            self.state.dirty = true;
                            return HandleResult::Consumed;
                        }
                    }
                }
            }
            return HandleResult::Ignored;
        };

        // Close prompt: y/n/c
        if self.close_prompt {
            use txv_core::event::KeyCode;
            match &key.code {
                KeyCode::Char('y') => {
                    self.close_prompt = false;
                    let content = self.editor.buffer.content();
                    let _ = crate::editor::save::save_file(&self.path, &content);
                    self.editor.buffer.mark_saved();
                    queue.put_command(
                        crate::commands::CM_FILE_CLOSED,
                        Some(Box::new(self.path.to_string_lossy().to_string())),
                    );
                    queue.put_command(CM_TAB_CLOSE, None);
                }
                KeyCode::Char('n') => {
                    self.close_prompt = false;
                    self.editor.buffer.mark_saved(); // discard
                    queue.put_command(
                        crate::commands::CM_FILE_CLOSED,
                        Some(Box::new(self.path.to_string_lossy().to_string())),
                    );
                    queue.put_command(CM_TAB_CLOSE, None);
                }
                _ => {
                    self.close_prompt = false;
                    self.editor.status = String::new();
                }
            }
            self.state.dirty = true;
            return HandleResult::Consumed;
        }

        let old_mode = self.editor.mode;
        let old_line = self.editor.cursor_line;
        let old_col = self.editor.cursor_col;

        if old_mode == crate::editor::keymap::EditorMode::Command
            || old_mode == crate::editor::keymap::EditorMode::Search
        {
            let result = self.handle_command_input(key, queue);
            self.emit_status_changes(old_mode, old_line, old_col, queue);
            return result;
        }

        let cmd = self.editor.keymap.handle_key(key, self.editor.mode);
        if cmd == crate::editor::command::Command::Noop {
            return HandleResult::Consumed;
        }

        let action = self.editor.execute(cmd);
        // Track edits for autosave
        if matches!(action, crate::editor::EditorAction::ContentChanged) {
            self.last_edit_tick = self.tick_counter;
        }
        self.handle_action(action, queue);
        self.ensure_cursor_visible();
        self.state.dirty = true;
        self.emit_status_changes(old_mode, old_line, old_col, queue);
        self.sync_title();
        HandleResult::Consumed
    }

    fn can_close(&self) -> CloseResult {
        if !self.editor.buffer.is_dirty() {
            return CloseResult::Ok;
        }
        if self.settings.autosave {
            return CloseResult::Ok; // will be saved on close
        }
        CloseResult::Denied("unsaved changes".to_string())
    }
}
