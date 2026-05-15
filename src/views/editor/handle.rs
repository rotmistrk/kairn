//! EditorView event handling helpers.

use txv_core::prelude::*;

use super::EditorView;
use crate::commands::*;
use crate::editor::EditorAction;

impl EditorView {
    pub(super) fn handle_command_input(
        &mut self,
        key: &txv_core::event::KeyEvent,
        queue: &mut EventQueue,
    ) -> HandleResult {
        use txv_core::event::KeyCode;
        // Ctrl+C cancels command/search mode; other Ctrl+keys pass through
        if key.modifiers.ctrl {
            if key.code == KeyCode::Char('c') {
                self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                self.editor.command_buf.clear();
                self.state.mark_dirty();
                return HandleResult::Consumed;
            }
            return HandleResult::Ignored;
        }
        match &key.code {
            KeyCode::Esc => {
                self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                self.editor.command_buf.clear();
            }
            KeyCode::Enter => {
                let buf = self.editor.command_buf.clone();
                if self.editor.mode == crate::editor::keymap::EditorMode::Search {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                    let action = self.editor.execute(crate::editor::command::Command::SearchForward(buf));
                    self.handle_action(action, queue);
                } else {
                    self.editor.mode = crate::editor::keymap::EditorMode::Normal;
                    let action = self.editor.execute(crate::editor::command::Command::ExCommand(buf));
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
            KeyCode::Tab => {
                self.complete_command_buf();
            }
            KeyCode::Char(c) => {
                self.editor.command_buf.push(*c);
            }
            _ => {}
        }
        self.ensure_cursor_visible();
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    pub(super) fn handle_action(&mut self, action: EditorAction, queue: &mut EventQueue) {
        match action {
            EditorAction::SaveRequested => {
                let name = self.path.file_name().unwrap_or(self.path.as_os_str()).to_os_string();
                if self.save_buffer() {
                    queue.put_command(CM_SAVE, None);
                    let msg = txv_core::message::Message::info("editor", format!("Saved: {}", name.to_string_lossy()));
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                } else {
                    let msg = txv_core::message::Message::error(
                        "editor",
                        format!("Failed to save: {}", name.to_string_lossy()),
                    );
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
            }
            EditorAction::CloseRequested => {
                if self.editor.buffer.is_dirty() && !self.settings.autosave {
                    self.eviction_close = false;
                    let path = self.path.to_string_lossy().to_string();
                    let ctx = crate::commands::ConfirmContext::EditorClose(path);
                    queue.put_command(crate::commands::CM_SET_CONFIRM_CONTEXT, Some(Box::new(ctx)));
                    queue.put_command(
                        crate::commands::CM_CONFIRM,
                        Some(Box::new("Save changes? [y]es [n]o [Esc]cancel".to_string())),
                    );
                    self.state.mark_dirty();
                } else {
                    if self.settings.autosave && self.editor.buffer.is_dirty() {
                        self.save_buffer();
                    }
                    let p = self.path.to_string_lossy().to_string();
                    queue.put_command(crate::commands::CM_FILE_CLOSED, Some(Box::new(p)));
                    queue.put_command(CM_TAB_CLOSE, None);
                }
            }
            EditorAction::ForceCloseRequested => {
                self.editor.buffer.mark_saved();
                let p = self.path.to_string_lossy().to_string();
                queue.put_command(crate::commands::CM_FILE_CLOSED, Some(Box::new(p)));
                queue.put_command(CM_TAB_CLOSE, None);
            }
            EditorAction::ShellOutput(output) => {
                queue.put_command(crate::commands::CM_SHELL_OUTPUT, Some(Box::new(output)));
            }
            EditorAction::OpenFile(filename) => {
                let cmd = format!("e {filename}");
                queue.put_command(crate::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            EditorAction::SetGlobal(opt) => {
                queue.put_command(CM_SET_GLOBAL, Some(Box::new(opt)));
            }
            EditorAction::Diff(args) => {
                self.toggle_diff(&args);
                if !self.editor.status.is_empty() {
                    let msg = txv_core::message::Message::info("editor", self.editor.status.clone());
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
                let mode = if self.in_diff_mode() {
                    "DIFF"
                } else {
                    "NOR"
                };
                queue.put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new(mode.to_string())));
            }
            EditorAction::NoDiff => {
                self.exit_diff();
                queue.put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
            }
            EditorAction::LspGotoDefinition => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                queue.put_command(crate::commands::CM_LSP_GOTO_DEF, Some(Box::new(data)));
            }
            EditorAction::LspFindReferences => {
                let word = self.editor.word_under_cursor().unwrap_or_default();
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                    word,
                );
                queue.put_command(crate::commands::CM_LSP_FIND_REFS, Some(Box::new(data)));
            }
            EditorAction::LspHover => {
                let data = (
                    self.path.clone(),
                    self.editor.cursor_line as u32,
                    self.editor.cursor_col as u32,
                );
                queue.put_command(crate::commands::CM_LSP_HOVER, Some(Box::new(data)));
            }
            EditorAction::AppCommand(cmd) => {
                queue.put_command(crate::commands::CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
            }
            _ => {}
        }
    }

    pub(super) fn ensure_cursor_visible(&mut self) {
        let h = self.state.bounds().h as usize;
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

    pub(super) fn emit_status_changes(
        &self,
        old_mode: crate::editor::keymap::EditorMode,
        old_line: usize,
        old_col: usize,
        queue: &mut EventQueue,
    ) {
        use crate::commands::{CM_CURSOR_MOVED, CM_MODE_CHANGED};
        use txv_widgets::CursorPos;

        if self.editor.mode != old_mode {
            let name = match self.editor.mode {
                crate::editor::keymap::EditorMode::Normal => "NOR",
                crate::editor::keymap::EditorMode::Insert => "INS",
                crate::editor::keymap::EditorMode::Visual | crate::editor::keymap::EditorMode::VisualLine => "VIS",
                crate::editor::keymap::EditorMode::Command => "CMD",
                crate::editor::keymap::EditorMode::Search => "CMD",
            };
            queue.put_command(CM_MODE_CHANGED, Some(Box::new(name.to_string())));
        }
        if self.editor.cursor_line != old_line || self.editor.cursor_col != old_col {
            let pos = CursorPos {
                line: (self.editor.cursor_line + 1) as u32,
                col: (self.editor.cursor_col + 1) as u32,
            };
            queue.put_command(CM_CURSOR_MOVED, Some(Box::new(pos)));
        }
        // Emit diagnostic message if cursor is on a diagnostic line
        if self.editor.cursor_line != old_line {
            let msg = self.diagnostic_at_cursor().map(|s| s.to_string()).unwrap_or_default();
            queue.put_command(crate::commands::CM_DIAGNOSTIC, Some(Box::new(msg)));
        }
    }

    /// Update display_title based on dirty state.
    pub fn sync_title(&mut self) {
        let name = self.path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        if self.editor.buffer.is_dirty() {
            self.display_title = format!("*{name}");
        } else {
            self.display_title = name.to_string();
        }
    }

    /// Emit hook triggers for char-inserted and word-completed events.
    pub(super) fn emit_hook_triggers(&self, cmd: &crate::editor::command::Command, queue: &mut EventQueue) {
        use crate::editor::command::Command;
        if let Command::InsertChar(ch) = cmd {
            queue.put_command(crate::commands::CM_CHAR_INSERTED, Some(Box::new(*ch)));
            // Check for word-completed: space/punctuation after a word
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                // Look back for the word that just ended
                if let Some(word) = self.word_before_cursor() {
                    queue.put_command(crate::commands::CM_WORD_COMPLETED, Some(Box::new(word)));
                }
            }
        }
    }

    /// Get the word immediately before the cursor (before the just-typed char).
    fn word_before_cursor(&self) -> Option<String> {
        let line = self.editor.buffer.line(self.editor.cursor_line)?;
        let chars: Vec<char> = line.chars().collect();
        // cursor_col points after the just-inserted char, so word ends at col-2
        let end = self.editor.cursor_col.checked_sub(1)?;
        if end == 0 || !chars.get(end.checked_sub(1)?)?.is_alphanumeric() {
            return None;
        }
        let word_end = end;
        let start = (0..word_end)
            .rev()
            .take_while(|&i| chars.get(i).is_some_and(|c| c.is_alphanumeric() || *c == '_'))
            .count();
        let begin = word_end - start;
        if begin == word_end {
            return None;
        }
        Some(chars[begin..word_end].iter().collect())
    }
}
