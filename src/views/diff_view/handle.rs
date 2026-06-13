//! DiffView key handling — navigation, exit, revert.

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::prelude::*;

use crate::commands::{CM_DIFF_EXIT, CM_DIFF_REVERT, CM_MODE_CHANGED, CM_SAVE_ALL};
use crate::views::editor::diff_model::is_change;

use super::DiffView;

impl DiffView {
    pub(super) fn handle_key(&mut self, key: &KeyEvent) -> HandleResult {
        if self.cmd_active {
            return self.handle_cmd_input(key);
        }
        match key.code() {
            KeyCode::Char(':') => self.enter_cmd_mode(),
            KeyCode::Esc | KeyCode::Char('q') => self.request_exit(),
            KeyCode::Enter => self.request_exit(),
            KeyCode::Char('R') => self.do_revert(),
            _ => return self.handle_nav_key(key),
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn handle_nav_key(&mut self, key: &KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Char('n') => self.next_hunk(),
            KeyCode::Char('N') => self.prev_hunk(),
            KeyCode::Char('j') | KeyCode::Down => self.move_cursor(1),
            KeyCode::Char('k') | KeyCode::Up => self.move_cursor(-1),
            KeyCode::Char('G') => self.move_end(),
            KeyCode::Char('g') => self.move_start(),
            KeyCode::PageDown | KeyCode::Char(' ') => self.move_cursor(self.height() as i32 - 1),
            KeyCode::PageUp => self.move_cursor(-(self.height() as i32 - 1)),
            _ => return HandleResult::Ignored,
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn enter_cmd_mode(&mut self) {
        self.cmd_active = true;
        self.cmd_buf.clear();
    }

    fn do_revert(&mut self) {
        let cursor = self.ds.cursor;
        self.state.put_command(
            CM_DIFF_EXIT,
            Some(Box::new((self.path.clone(), self.cursor_buf_line() as u32))),
        );
        self.state.put_command(CM_DIFF_REVERT, Some(Box::new(cursor)));
    }

    fn request_exit(&mut self) {
        let line = self.cursor_buf_line() as u32;
        self.state
            .put_command(CM_DIFF_EXIT, Some(Box::new((self.path.clone(), line))));
        self.state
            .put_command(CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
    }

    fn next_hunk(&mut self) {
        let start = self.ds.cursor + 1;
        if let Some(p) = self.ds.lines[start..].iter().position(is_change) {
            self.ds.cursor = start + p;
            self.ensure_visible();
        }
    }

    fn prev_hunk(&mut self) {
        if self.ds.cursor == 0 {
            return;
        }
        if let Some(p) = self.ds.lines[..self.ds.cursor].iter().rposition(is_change) {
            self.ds.cursor = p;
            self.ensure_visible();
        }
    }

    fn move_cursor(&mut self, delta: i32) {
        let max = self.ds.lines.len().saturating_sub(1);
        self.ds.cursor = (self.ds.cursor as i32 + delta).clamp(0, max as i32) as usize;
        self.ensure_visible();
    }

    fn move_end(&mut self) {
        self.ds.cursor = self.ds.lines.len().saturating_sub(1);
        self.ensure_visible();
    }

    fn move_start(&mut self) {
        self.ds.cursor = 0;
        self.ensure_visible();
    }

    fn ensure_visible(&mut self) {
        self.ds.ensure_visible(self.height());
    }

    fn handle_cmd_input(&mut self, key: &KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Esc => self.cmd_active = false,
            KeyCode::Enter => {
                self.cmd_active = false;
                self.execute_cmd();
            }
            KeyCode::Backspace => {
                self.cmd_buf.pop();
            }
            KeyCode::Char(ch) => self.cmd_buf.push(ch),
            _ => {}
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }

    fn execute_cmd(&mut self) {
        let cmd = self.cmd_buf.trim().to_string();
        if cmd.len() >= 3 && "revert".starts_with(&cmd) {
            self.do_revert();
        } else if cmd == "q" || (cmd.len() >= 3 && "nodiff".starts_with(&cmd)) {
            self.request_exit();
        } else if cmd == "w" {
            self.state.put_command(
                CM_DIFF_EXIT,
                Some(Box::new((self.path.clone(), self.cursor_buf_line() as u32))),
            );
            self.state.put_command(CM_SAVE_ALL, None);
        }
    }
}
