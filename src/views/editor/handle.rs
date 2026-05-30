//! EditorView event handling helpers.

use txv_core::prelude::*;

use super::EditorView;
use crate::commands::{CM_CHAR_INSERTED, CM_DIAGNOSTIC, CM_WORD_COMPLETED};
use crate::editor::command::Command;
use crate::editor::highlight_state::HighlightState;
use crate::editor::keymap::EditorMode;

pub(super) fn is_search_navigation(cmd: &Command) -> bool {
    matches!(
        cmd,
        Command::SearchNext | Command::SearchPrev | Command::SearchWordForward | Command::SearchWordBackward
    )
}

impl EditorView {
    pub(super) fn handle_command_input(&mut self, key: &txv_core::event::KeyEvent) -> HandleResult {
        use txv_core::event::KeyCode;
        if key.modifiers.ctrl {
            if key.code == KeyCode::Char('c') {
                self.editor.mode = EditorMode::Normal;
                self.editor.command_buf.clear();
                self.editor.highlight = None;
                self.state.mark_dirty();
                return HandleResult::Consumed;
            }
            return HandleResult::Ignored;
        }
        match &key.code {
            KeyCode::Esc => self.cancel_command_input(),
            KeyCode::Enter => self.submit_command_input(),
            KeyCode::Backspace => self.backspace_command_input(),
            KeyCode::Tab => self.complete_command_buf(),
            KeyCode::Up => self.history_prev(),
            KeyCode::Down => self.history_next(),
            KeyCode::Char(c) => {
                self.editor.command_buf.push(*c);
                self.editor.history_index = None;
                self.update_incsearch();
            }
            _ => {}
        }
        self.ensure_cursor_visible();
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn cancel_command_input(&mut self) {
        let is_search = self.editor.mode == EditorMode::Search;
        self.editor.mode = EditorMode::Normal;
        self.editor.command_buf.clear();
        if is_search {
            self.editor.highlight = None;
        }
    }

    fn submit_command_input(&mut self) {
        let buf = self.editor.command_buf.clone();
        if self.editor.mode == EditorMode::Search {
            if !buf.is_empty() {
                self.editor.command_history.push(buf.clone());
            }
            self.editor.mode = EditorMode::Normal;
            let action = self.editor.execute(Command::SearchForward(buf));
            self.handle_action(action);
        } else {
            if !buf.is_empty() {
                self.editor.command_history.push(buf.clone());
            }
            self.editor.mode = EditorMode::Normal;
            let action = self.editor.execute(Command::ExCommand(buf));
            self.handle_action(action);
        }
        self.editor.command_buf.clear();
        self.editor.history_index = None;
    }

    fn backspace_command_input(&mut self) {
        if self.editor.command_buf.is_empty() {
            self.editor.mode = EditorMode::Normal;
            self.editor.highlight = None;
        } else {
            self.editor.command_buf.pop();
            self.editor.history_index = None;
            self.update_incsearch();
        }
    }

    pub(super) fn emit_status_changes(&self, old_mode: EditorMode, old_line: usize, old_col: usize) {
        use crate::commands::{CM_CURSOR_MOVED, CM_MODE_CHANGED};
        use txv_widgets::CursorPos;

        if self.editor.mode != old_mode {
            let name = match self.editor.mode {
                EditorMode::Normal => "NOR",
                EditorMode::Insert => "INS",
                EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock => "VIS",
                EditorMode::Command => "CMD",
                EditorMode::Search => "CMD",
            };
            self.state
                .put_command(CM_MODE_CHANGED, Some(Box::new(name.to_string())));
        }
        if self.editor.cursor_line != old_line || self.editor.cursor_col != old_col {
            let pos = CursorPos::new(
                (self.editor.cursor_line + 1) as u32,
                (self.editor.cursor_col + 1) as u32,
            );
            self.state.put_command(CM_CURSOR_MOVED, Some(Box::new(pos)));
        }
        // Emit diagnostic message if cursor is on a diagnostic line
        if self.editor.cursor_line != old_line {
            let msg = self.diagnostic_at_cursor().map(|s| s.to_string()).unwrap_or_default();
            self.state.put_command(CM_DIAGNOSTIC, Some(Box::new(msg)));
        }
    }

    /// Update display_title based on dirty state.
    pub fn sync_title(&mut self) {
        let name = self.path.file_name().and_then(|n| n.to_str()).unwrap_or("untitled");
        self.display_title = name.to_string();
    }

    /// Emit hook triggers for char-inserted and word-completed events.
    pub(super) fn emit_hook_triggers(&self, cmd: &Command) {
        if let Command::InsertChar(ch) = cmd {
            self.state.put_command(CM_CHAR_INSERTED, Some(Box::new(*ch)));
            // Check for word-completed: space/punctuation after a word
            if ch.is_whitespace() || ch.is_ascii_punctuation() {
                // Look back for the word that just ended
                if let Some(word) = self.word_before_cursor() {
                    self.state.put_command(CM_WORD_COMPLETED, Some(Box::new(word)));
                }
            }
        }
    }

    /// Get the word immediately before the cursor (before the just-typed char).
    fn word_before_cursor(&self) -> Option<String> {
        let line = self.editor.buf().line(self.editor.cursor_line)?;
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

    /// Update incremental search highlights while typing in search mode.
    fn update_incsearch(&mut self) {
        if self.editor.mode != EditorMode::Search {
            return;
        }
        if !self.editor.options.incsearch {
            return;
        }
        let pattern = &self.editor.command_buf;
        if pattern.is_empty() {
            self.editor.highlight = None;
            return;
        }
        let content = self.editor.buf().content();
        let cursor_off = self
            .editor
            .buf()
            .line_col_to_offset(self.editor.cursor_line, self.editor.cursor_col)
            .unwrap_or(0);
        self.editor.highlight = HighlightState::build(pattern, &content, cursor_off);
    }
}
