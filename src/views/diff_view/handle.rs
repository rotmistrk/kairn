//! DiffView key handling — navigation, exit, revert, command line.

use txv_core::event::{KeyCode, KeyEvent};
use txv_core::prelude::*;

use crate::commands::{CM_DIFF_EXIT, CM_DIFF_REVERT, CM_MODE_CHANGED};
use crate::views::editor::diff_model::is_change;

use super::DiffView;

impl DiffView {
    pub(super) fn handle_key(&mut self, key: &KeyEvent) -> HandleResult {
        match key.code() {
            KeyCode::Char(':') => {
                self.activate_cmdline();
                return HandleResult::Consumed;
            }
            KeyCode::Esc | KeyCode::Char('q') => self.request_exit(),
            KeyCode::Enter => self.request_exit(),
            KeyCode::Char('R') => self.do_revert(),
            _ => return self.handle_nav_key(key),
        }
        self.group.mark_dirty();
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
            KeyCode::PageDown | KeyCode::Char(' ') => self.move_cursor(self.content_height() as i32 - 1),
            KeyCode::PageUp => self.move_cursor(-(self.content_height() as i32 - 1)),
            _ => return HandleResult::Ignored,
        }
        self.group.mark_dirty();
        HandleResult::Consumed
    }

    pub(super) fn handle_cmdline_event(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            return self.handle_cmdline_command(*id, data);
        }
        let result = self.group.dispatch(event);
        if let Some(r) = self.drain_child_commands() {
            return r;
        }
        self.group.mark_dirty();
        result
    }

    fn drain_child_commands(&mut self) -> Option<HandleResult> {
        let sink = self.group.sink()?;
        for ev in sink.drain() {
            if let Event::Command { id, data, .. } = ev {
                let r = self.handle_cmdline_command(id, &data);
                if r != HandleResult::Ignored {
                    return Some(r);
                }
            }
        }
        None
    }

    fn handle_cmdline_command(&mut self, id: CommandId, data: &Option<Box<dyn std::any::Any + Send>>) -> HandleResult {
        match id {
            CM_OK => {
                let text = data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<String>())
                    .cloned()
                    .unwrap_or_default();
                self.cmdline_submit(&text);
                HandleResult::Consumed
            }
            CM_CANCEL => {
                self.deactivate_cmdline();
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn cmdline_submit(&mut self, text: &str) {
        self.deactivate_cmdline();
        let cmd = text.trim();
        if cmd.len() >= 3 && "revert".starts_with(cmd) {
            self.do_revert();
        } else if cmd == "q" || (cmd.len() >= 3 && "nodiff".starts_with(cmd)) {
            self.request_exit();
        }
    }

    fn do_revert(&mut self) {
        let cursor = self.ds.cursor;
        self.group.put_command(
            CM_DIFF_EXIT,
            Some(Box::new((self.path.clone(), self.cursor_buf_line() as u32))),
        );
        self.group.put_command(CM_DIFF_REVERT, Some(Box::new(cursor)));
    }

    fn request_exit(&mut self) {
        let line = self.cursor_buf_line() as u32;
        self.group
            .put_command(CM_DIFF_EXIT, Some(Box::new((self.path.clone(), line))));
        self.group
            .put_command(CM_MODE_CHANGED, Some(Box::new("NOR".to_string())));
    }

    fn next_hunk(&mut self) {
        let start = self.ds.cursor + 1;
        if start < self.ds.lines.len() {
            if let Some(p) = self.ds.lines[start..].iter().position(is_change) {
                self.ds.cursor = start + p;
                self.ensure_visible();
            }
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
        self.ds.ensure_visible(self.content_height());
    }
}
