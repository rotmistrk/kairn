//! EditorView tick handling: autosave, completion trigger, disk change detection.

use std::fs::{metadata, read_dir, read_to_string};

use txv_core::message::Message;

use super::EditorView;
use crate::commands::{
    ConfirmContext, ContentChanged, CM_CONFIRM, CM_CONTENT_CHANGED, CM_LSP_COMPLETION, CM_LSP_SIGNATURE_HELP,
    CM_SET_CONFIRM_CONTEXT,
};
use crate::editor::keymap::EditorMode;

impl EditorView {
    /// Handle tick event: autosave + completion trigger + LSP didChange.
    pub(super) fn handle_tick(&mut self) {
        self.tick_counter += 1;
        if self.tick_counter.is_multiple_of(20) {
            self.check_disk_change();
        }
        // LSP didChange: 3 ticks after last edit (debounced)
        if self.last_edit_tick > 0 && self.tick_counter - self.last_edit_tick == 3 {
            let changed = ContentChanged {
                path: self.path.clone(),
                content: self.editor.buf().content(),
            };
            self.state.put_command(CM_CONTENT_CHANGED, Some(Box::new(changed)));
        }
        // Completion trigger: 5 ticks after last edit in insert mode
        if self.editor.mode() == EditorMode::Insert
            && self.last_edit_tick > 0
            && self.tick_counter - self.last_edit_tick == 5
        {
            let pos = (
                self.path.clone(),
                self.editor.cursor_line() as u32,
                self.editor.cursor_col() as u32,
            );
            self.state.put_command(CM_LSP_COMPLETION, Some(Box::new(pos.clone())));
            if self.is_inside_call() {
                self.state.put_command(CM_LSP_SIGNATURE_HELP, Some(Box::new(pos)));
            }
        }
        if self.settings.autosave
            && self.last_edit_tick > 0
            && self.tick_counter - self.last_edit_tick >= self.settings.autosave_delay as u64
        {
            self.last_edit_tick = 0;
            if self.editor.buf().is_dirty() && self.save_buffer() {
                self.sync_title();
            }
        }
    }

    /// Save buffer via the configured store. Returns true on success.
    pub(super) fn save_buffer(&mut self) -> bool {
        let content = self.editor.buf().content();
        if self.store.save(&content).is_ok() {
            self.editor.buf().mark_saved();
            self.disk_mtime = metadata(&self.path).and_then(|m| m.modified()).ok();
            self.refresh_gutter_signs();
            true
        } else {
            false
        }
    }

    /// Check if the file was modified externally. Auto-reload if clean, prompt if dirty.
    fn check_disk_change(&mut self) {
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
        if self.editor.buf().is_dirty() {
            let path = self.path.to_string_lossy().to_string();
            let ctx = ConfirmContext::FileReload(path);
            self.state.put_command(CM_SET_CONFIRM_CONTEXT, Some(Box::new(ctx)));
            let prompt = format!("{} changed on disk — reload? [y/n]", self.display_title);
            self.state.put_command(CM_CONFIRM, Some(Box::new(prompt)));
        } else if let Ok(content) = read_to_string(&self.path) {
            self.editor.replace_content(&content);
            self.hl_cache.borrow_mut().invalidate_all();
            self.state.mark_dirty();
            let msg = Message::info("editor", format!("{} reloaded", self.display_title));
            self.state
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }

    pub(super) fn complete_command_buf(&mut self) {
        let buf = self.editor.command_buf().to_string();
        if buf.starts_with("e ") || buf.starts_with("edit ") {
            self.complete_command_path();
            return;
        }
        if buf.starts_with("set ") || buf.starts_with("setglobal ") {
            self.complete_set_option(&buf);
            return;
        }
        let app_commands = Self::app_command_names();
        let extra: Vec<&str> = app_commands.iter().map(|s| s.as_str()).collect();
        self.editor.complete_ex(&extra);
    }

    fn complete_set_option(&mut self, buf: &str) {
        use crate::handler_set::SET_OPTIONS;
        let (prefix, sub) = if let Some(s) = buf.strip_prefix("setglobal ") {
            ("setglobal ", s)
        } else {
            ("set ", buf.strip_prefix("set ").unwrap_or(""))
        };
        let matches: Vec<&str> = SET_OPTIONS
            .iter()
            .filter(|o| o.name.starts_with(sub))
            .map(|o| o.name)
            .collect();
        if matches.len() == 1 {
            self.editor.set_command_buf(format!("{prefix}{}", matches[0]));
        } else if matches.len() > 1 {
            // Complete common prefix
            let common = common_prefix(&matches);
            if common.len() > sub.len() {
                self.editor.set_command_buf(format!("{prefix}{common}"));
            }
        }
    }

    fn app_command_names() -> Vec<String> {
        use crate::handler_exec::dispatch_table;
        dispatch_table()
            .flat_map(|e| e.names.iter().copied())
            .map(|s| s.to_string())
            .collect()
    }

    fn complete_command_path(&mut self) {
        use std::path::Path;
        let buf = self.editor.command_buf();
        let partial = buf
            .strip_prefix("e ")
            .or_else(|| buf.strip_prefix("edit "))
            .unwrap_or("");
        let (search_dir, file_prefix, dir_prefix) = if partial.contains('/') {
            let p = Path::new(partial);
            let parent = p.parent().unwrap_or(Path::new(""));
            let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let dp = format!("{}/", parent.display());
            (self.root_dir.join(parent), prefix.to_string(), dp)
        } else {
            (self.root_dir.clone(), partial.to_string(), String::new())
        };
        let Ok(entries) = read_dir(&search_dir) else {
            return;
        };
        let mut matches: Vec<String> = Vec::new();
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy().to_string();
            if name_str.starts_with(&file_prefix) {
                matches.push(format!("{dir_prefix}{name_str}"));
            }
        }
        if matches.len() == 1 {
            let prefix = if buf.starts_with("edit ") {
                "edit "
            } else {
                "e "
            };
            self.editor.set_command_buf(format!("{prefix}{}", matches[0]));
        }
    }
}

fn common_prefix(strings: &[&str]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = strings[0];
    let len = first
        .char_indices()
        .take_while(|&(i, c)| strings[1..].iter().all(|s| s.as_bytes().get(i) == Some(&(c as u8))))
        .map(|(i, c)| i + c.len_utf8())
        .last()
        .unwrap_or(0);
    first[..len].to_string()
}
