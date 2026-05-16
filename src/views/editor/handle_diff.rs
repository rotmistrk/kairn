//! Diff mode key handling — navigation, exit, jump.

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::prelude::*;

use super::EditorView;

impl EditorView {
    pub(super) fn handle_diff_key(&mut self, key: &KeyEvent, queue: &mut EventQueue) -> HandleResult {
        // Allow : to enter command mode (for :diff -U3, :nodiff, etc.)
        if key.code == KeyCode::Char(':') && !key.modifiers.ctrl {
            self.editor.mode = crate::editor::keymap::EditorMode::Command;
            self.editor.command_buf.clear();
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }

        // If in command/search mode, delegate to normal command input
        if self.editor.mode == crate::editor::keymap::EditorMode::Command
            || self.editor.mode == crate::editor::keymap::EditorMode::Search
        {
            let result = self.handle_command_input(key, queue);
            self.state.mark_dirty();
            return result;
        }

        match key.code {
            KeyCode::Esc => {
                self.exit_diff();
                queue.put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
                queue.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(txv_core::message::Message::info("editor", "Exited diff mode"))),
                );
            }
            KeyCode::Enter => {
                self.exit_diff_at_cursor();
                queue.put_command(crate::commands::CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
                queue.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(txv_core::message::Message::info("editor", "Exited diff mode"))),
                );
            }
            KeyCode::Char('n') => self.diff_next_hunk(),
            KeyCode::Char('N') => self.diff_prev_hunk(),
            KeyCode::Char('j') | KeyCode::Down => self.diff_move(1),
            KeyCode::Char('k') | KeyCode::Up => self.diff_move(-1),
            KeyCode::Char('G') => self.diff_move_end(),
            KeyCode::Char('g') => self.diff_move_start(),
            KeyCode::PageDown | KeyCode::Char(' ') => {
                let h = self.state.bounds().h as i32;
                self.diff_move(h - 1);
            }
            KeyCode::PageUp => {
                let h = self.state.bounds().h as i32;
                self.diff_move(-(h - 1));
            }
            KeyCode::Char('R') => {
                let msg = match self.revert_hunk() {
                    Ok(m) => txv_core::message::Message::info("editor", m),
                    Err(e) => txv_core::message::Message::error("editor", e),
                };
                queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
            KeyCode::Char('/') => {
                self.editor.mode = crate::editor::keymap::EditorMode::Search;
                self.editor.command_buf.clear();
            }
            _ => {}
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn diff_next_hunk(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            if let Some(pos) = ds.next_hunk() {
                ds.cursor = pos;
                ds.ensure_visible(self.state.bounds().h as usize);
            }
        }
    }

    fn diff_prev_hunk(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            if let Some(pos) = ds.prev_hunk() {
                ds.cursor = pos;
                ds.ensure_visible(self.state.bounds().h as usize);
            }
        }
    }

    fn diff_move(&mut self, delta: i32) {
        if let Some(ds) = &mut self.diff_state {
            let max = ds.lines.len().saturating_sub(1);
            let new = (ds.cursor as i32 + delta).clamp(0, max as i32) as usize;
            ds.cursor = new;
            ds.ensure_visible(self.state.bounds().h as usize);
        }
    }

    fn diff_move_end(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            ds.cursor = ds.lines.len().saturating_sub(1);
            ds.ensure_visible(self.state.bounds().h as usize);
        }
    }

    fn diff_move_start(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            ds.cursor = 0;
            ds.ensure_visible(self.state.bounds().h as usize);
        }
    }
}
