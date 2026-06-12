//! Tick handling: autosave, completion trigger, disk change detection.

use std::fs::{metadata, read_to_string};

use txv_core::message::Message;
use txv_core::prelude::HandleResult;

use super::delegate::KairnDelegate;
use crate::commands::{
    ConfirmContext, ContentChanged, CM_CONFIRM, CM_CONTENT_CHANGED, CM_LSP_COMPLETION, CM_LSP_SIGNATURE_HELP,
    CM_SET_CONFIRM_CONTEXT,
};
use crate::editor::keymap::EditorMode;
use crate::editor::Editor;

impl KairnDelegate {
    pub(crate) fn handle_tick(&mut self, editor: &mut Editor, tick: u64) -> HandleResult {
        self.current_tick = tick;
        // If an edit happened since last tick, record this tick as the edit time
        if self.last_edit_tick == u64::MAX {
            self.last_edit_tick = tick;
        }
        if tick.is_multiple_of(20) {
            self.check_disk_change(editor);
        }
        // LSP didChange: 3 ticks after last edit
        if self.last_edit_tick > 0 && tick - self.last_edit_tick == 3 {
            let changed = ContentChanged {
                path: self.path.clone(),
                content: editor.buf().content(),
            };
            self.emit(CM_CONTENT_CHANGED, Some(Box::new(changed)));
        }
        // Completion trigger: 5 ticks after last edit in insert mode
        if editor.mode() == EditorMode::Insert && self.last_edit_tick > 0 && tick - self.last_edit_tick == 5 {
            let pos = (
                self.path.clone(),
                editor.cursor_line() as u32,
                editor.cursor_col() as u32,
            );
            self.emit(CM_LSP_COMPLETION, Some(Box::new(pos.clone())));
            if self.is_inside_call(editor) {
                self.emit(CM_LSP_SIGNATURE_HELP, Some(Box::new(pos)));
            }
        }
        self.check_autosave(editor, tick);
        HandleResult::Ignored
    }

    fn check_autosave(&mut self, editor: &mut Editor, tick: u64) {
        if !self.settings.autosave || self.last_edit_tick == 0 {
            return;
        }
        if tick - self.last_edit_tick < self.settings.autosave_delay as u64 {
            return;
        }
        self.last_edit_tick = 0;
        if editor.buf().is_dirty() {
            let content = editor.buf().content();
            if self.store.save(&content).is_ok() {
                editor.buf().mark_saved();
                self.disk_mtime = metadata(&self.path).and_then(|m| m.modified()).ok();
                self.dirty = true;
            }
        }
    }

    fn check_disk_change(&mut self, editor: &mut Editor) {
        let Some(known_mtime) = self.disk_mtime else {
            return;
        };
        let Ok(meta) = metadata(&self.path) else {
            return;
        };
        let Ok(current_mtime) = meta.modified() else {
            return;
        };
        if current_mtime == known_mtime {
            return;
        }
        self.disk_mtime = Some(current_mtime);
        if editor.buf().is_dirty() {
            let path = self.path.to_string_lossy().to_string();
            let ctx = ConfirmContext::FileReload(path);
            self.emit(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ctx)));
            let prompt = format!("{} changed on disk — reload? [y/n]", self.display_title);
            self.emit(CM_CONFIRM, Some(Box::new(prompt)));
        } else if let Ok(content) = read_to_string(&self.path) {
            if content == editor.buf().content() {
                return;
            }
            let old_line = editor.cursor_line();
            let old_col = editor.cursor_col();
            let old_scroll = editor.viewport_scroll();
            editor.replace_content(&content);
            let max_line = editor.buf().line_count().saturating_sub(1);
            editor.set_cursor_line(old_line.min(max_line));
            let line_len = editor.buf().line(old_line.min(max_line)).map(|l| l.len()).unwrap_or(0);
            editor.set_cursor_col(old_col.min(line_len));
            editor.set_viewport_scroll(old_scroll.min(max_line));
            self.dirty = true;
            let msg = Message::info("editor", format!("{} reloaded", self.display_title));
            self.emit(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }

    fn is_inside_call(&self, editor: &Editor) -> bool {
        let line = editor.buf().line(editor.cursor_line()).unwrap_or_default();
        let before = &line[..editor.cursor_col().min(line.len())];
        let mut depth: i32 = 0;
        for ch in before.chars() {
            match ch {
                '(' => depth += 1,
                ')' => depth -= 1,
                _ => {}
            }
        }
        depth > 0
    }
}
