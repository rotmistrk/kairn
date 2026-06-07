//! Diff mode key handling — navigation, exit, jump.

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::message::Message;
use txv_core::prelude::*;

use super::EditorView;
use crate::commands::CM_MODE_CHANGED;
use crate::editor::keymap::EditorMode;

impl EditorView {
    pub(super) fn handle_diff_key(&mut self, key: &KeyEvent) -> HandleResult {
        if key.code() == KeyCode::Char(':') && !key.modifiers().ctrl() {
            self.editor.mode = EditorMode::Command;
            self.editor.command_buf.clear();
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }

        if self.editor.mode == EditorMode::Command || self.editor.mode == EditorMode::Search {
            let result = self.handle_command_input(key);
            self.state.mark_dirty();
            return result;
        }

        self.dispatch_diff_key(key);
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn dispatch_diff_key(&mut self, key: &KeyEvent) {
        match key.code() {
            KeyCode::Esc => self.exit_diff_with_message(),
            KeyCode::Enter => {
                self.exit_diff_at_cursor();
                self.state
                    .put_command(CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
                self.state.put_command(
                    txv_widgets::CM_STATUS_MESSAGE,
                    Some(Box::new(Message::info("editor", "Exited diff mode"))),
                );
            }
            KeyCode::Char('n') => self.diff_next_hunk(),
            KeyCode::Char('N') => self.diff_prev_hunk(),
            KeyCode::Char('j') | KeyCode::Down => self.diff_move(1),
            KeyCode::Char('k') | KeyCode::Up => self.diff_move(-1),
            KeyCode::Char('G') => self.diff_move_end(),
            KeyCode::Char('g') => self.diff_move_start(),
            KeyCode::PageDown | KeyCode::Char(' ') => {
                let h = self.state.bounds().h() as i32;
                self.diff_move(h - 1);
            }
            KeyCode::PageUp => {
                let h = self.state.bounds().h() as i32;
                self.diff_move(-(h - 1));
            }
            KeyCode::Char('R') => {
                let msg = match self.revert_hunk() {
                    Ok(m) => Message::info("editor", m),
                    Err(e) => Message::error("editor", e),
                };
                self.state
                    .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            }
            KeyCode::Char('/') => {
                self.editor.mode = EditorMode::Search;
                self.editor.command_buf.clear();
            }
            _ => {}
        }
    }

    fn exit_diff_with_message(&mut self) {
        self.exit_diff();
        self.state
            .put_command(CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
        self.state.put_command(
            txv_widgets::CM_STATUS_MESSAGE,
            Some(Box::new(Message::info("editor", "Exited diff mode"))),
        );
    }

    fn diff_next_hunk(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            if let Some(pos) = ds.next_hunk() {
                ds.cursor = pos;
                ds.ensure_visible(self.state.bounds().h() as usize);
            }
        }
    }

    fn diff_prev_hunk(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            if let Some(pos) = ds.prev_hunk() {
                ds.cursor = pos;
                ds.ensure_visible(self.state.bounds().h() as usize);
            }
        }
    }

    fn diff_move(&mut self, delta: i32) {
        if let Some(ds) = &mut self.diff_state {
            let max = ds.lines.len().saturating_sub(1);
            let new = (ds.cursor as i32 + delta).clamp(0, max as i32) as usize;
            ds.cursor = new;
            ds.ensure_visible(self.state.bounds().h() as usize);
        }
    }

    fn diff_move_end(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            ds.cursor = ds.lines.len().saturating_sub(1);
            ds.ensure_visible(self.state.bounds().h() as usize);
        }
    }

    fn diff_move_start(&mut self) {
        if let Some(ds) = &mut self.diff_state {
            ds.cursor = 0;
            ds.ensure_visible(self.state.bounds().h() as usize);
        }
    }

    pub(super) fn handle_sbs_key(&mut self, key: &KeyEvent) -> HandleResult {
        if key.code() == KeyCode::Char(':') && !key.modifiers().ctrl() {
            self.editor.mode = EditorMode::Command;
            self.editor.command_buf.clear();
            self.state.mark_dirty();
            return HandleResult::Consumed;
        }

        if self.editor.mode == EditorMode::Command || self.editor.mode == EditorMode::Search {
            let result = self.handle_command_input(key);
            self.state.mark_dirty();
            return result;
        }

        self.dispatch_sbs_key(key);
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn dispatch_sbs_key(&mut self, key: &KeyEvent) {
        match key.code() {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.sbs_state = None;
                self.state.mark_dirty();
                self.state
                    .put_command(CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
            }
            KeyCode::Char('/') => {
                self.editor.mode = EditorMode::Search;
                self.editor.command_buf.clear();
                self.state.mark_dirty();
            }
            KeyCode::Char('j') | KeyCode::Down => self.sbs_scroll(1),
            KeyCode::Char('k') | KeyCode::Up => self.sbs_scroll(-1),
            KeyCode::Char('G') => {
                if let Some(sbs) = &mut self.sbs_state {
                    sbs.scroll = sbs.left.len().saturating_sub(1);
                }
            }
            KeyCode::Char('g') => {
                if let Some(sbs) = &mut self.sbs_state {
                    sbs.scroll = 0;
                }
            }
            KeyCode::PageDown | KeyCode::Char(' ') => {
                let h = self.state.bounds().h() as i32;
                self.sbs_scroll(h - 1);
            }
            KeyCode::PageUp => {
                let h = self.state.bounds().h() as i32;
                self.sbs_scroll(-(h - 1));
            }
            _ => {}
        }
    }

    fn sbs_scroll(&mut self, delta: i32) {
        if let Some(sbs) = &mut self.sbs_state {
            let max = sbs.left.len().saturating_sub(1);
            let new = (sbs.scroll as i32 + delta).clamp(0, max as i32) as usize;
            sbs.scroll = new;
        }
    }
}
