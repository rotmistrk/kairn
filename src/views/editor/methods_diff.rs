//! EditorView diff mode + flush_pending.

use std::path::PathBuf;

use txv_core::message::Message;

use super::EditorView;
use crate::commands::{CM_DIFF_OPEN_VIEW, CM_FILE_CLOSED, CM_FS_CHANGED, CM_TAB_CLOSE};

impl EditorView {
    pub fn toggle_diff(&mut self, args: &str) {
        let dm = self.inner.delegate_mut();
        dm.pending_diff = Some(args.to_string());
    }

    pub fn set_diff_state(&mut self, state: super::diff_model::DiffState) {
        *self.inner.delegate_mut().diff_state_mut() = Some(state);
        let status = self.editor().status().to_string();
        if !status.is_empty() {
            let msg = Message::info("editor", status);
            self.inner
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        self.inner.mark_dirty();
    }

    pub fn set_sbs_state(&mut self, _state: super::sbs_model::SbsDiffState) {}

    pub fn revert_hunk(&mut self) -> Result<String, String> {
        use super::diff_model::is_change;
        let ds = self
            .inner
            .delegate()
            .diff_state_ref()
            .as_ref()
            .ok_or("Not in diff mode")?;
        let cursor = ds.cursor;
        if cursor >= ds.lines.len() {
            return Err("Cursor out of range".to_string());
        }
        if !is_change(&ds.lines[cursor]) {
            return Err("Cursor not on a change".to_string());
        }
        let (start, end) = Self::find_hunk_bounds(&ds.lines, cursor);
        let ds = self
            .inner
            .delegate()
            .diff_state_ref()
            .as_ref()
            .ok_or("Not in diff mode")?;
        let (buf_lines, deleted_text, insert_line) = Self::collect_hunk_data(ds, start, end);
        self.apply_revert(&buf_lines, &deleted_text, insert_line);
        *self.inner.delegate_mut().diff_state_mut() = None;
        self.inner.mark_dirty();
        Ok("Hunk reverted".to_string())
    }

    fn find_hunk_bounds(lines: &[super::diff_model::DiffLine], cursor: usize) -> (usize, usize) {
        use super::diff_model::is_change;
        let mut s = cursor;
        while s > 0 && is_change(&lines[s - 1]) {
            s -= 1;
        }
        let mut e = cursor + 1;
        while e < lines.len() && is_change(&lines[e]) {
            e += 1;
        }
        (s, e)
    }

    fn collect_hunk_data(
        ds: &super::diff_model::DiffState,
        start: usize,
        end: usize,
    ) -> (Vec<usize>, Vec<String>, usize) {
        use super::diff_model::DiffLine;
        let mut buf_lines: Vec<usize> = Vec::new();
        let mut deleted_text: Vec<String> = Vec::new();
        for line in &ds.lines[start..end] {
            match line {
                DiffLine::Added { buf_line } => buf_lines.push(*buf_line),
                DiffLine::Deleted { text, .. } => deleted_text.push(text.clone()),
                _ => {}
            }
        }
        let insert_line = if buf_lines.is_empty() && start > 0 {
            match &ds.lines[start - 1] {
                DiffLine::Context { buf_line, .. } | DiffLine::Added { buf_line } => buf_line + 1,
                _ => 0,
            }
        } else {
            0
        };
        (buf_lines, deleted_text, insert_line)
    }

    fn apply_revert(&mut self, buf_lines: &[usize], deleted_text: &[String], insert_line: usize) {
        let mut buf = self.editor_mut().buf();
        buf.begin_group();
        if !buf_lines.is_empty() {
            let first = buf_lines[0];
            let last = buf_lines[buf_lines.len() - 1];
            let start_off = buf.line_col_to_offset(first, 0).unwrap_or(0);
            let end_off = if last + 1 < buf.line_count() {
                buf.line_col_to_offset(last + 1, 0).unwrap_or(buf.len())
            } else {
                buf.len()
            };
            if end_off > start_off {
                buf.delete(start_off, end_off);
            }
            if !deleted_text.is_empty() {
                let insert = deleted_text.join("\n") + "\n";
                let off = buf.line_col_to_offset(first, 0).unwrap_or(buf.len());
                buf.insert(off, &insert);
            }
        } else if !deleted_text.is_empty() {
            let insert = deleted_text.join("\n") + "\n";
            let off = buf.line_col_to_offset(insert_line, 0).unwrap_or(buf.len());
            buf.insert(off, &insert);
        }
        buf.end_group();
        drop(buf);
        self.editor_mut().clamp_cursor();
    }

    /// Drain pending commands from delegate and emit them on the view.
    pub(crate) fn flush_pending(&mut self) {
        self.flush_force_close();
        self.flush_diff();
        self.flush_revert();
        self.flush_nodiff();
        self.flush_save();
        self.flush_sync_settings();
        self.flush_commands();
    }

    fn flush_force_close(&mut self) {
        if !self.inner.delegate().is_force_close() {
            return;
        }
        self.inner.delegate_mut().set_force_close(false);
        self.editor_mut().buf().mark_saved();
        let p = self.path().to_string_lossy().to_string();
        self.inner.put_command(CM_FILE_CLOSED, Some(Box::new(p)));
        self.inner.put_command(CM_TAB_CLOSE, None);
    }

    fn flush_diff(&mut self) {
        if self.inner.delegate().pending_diff_ref().is_none() {
            return;
        }
        let args = self.inner.delegate_mut().take_pending_diff().unwrap_or_default();
        use crate::diff::git_file_content;
        use crate::views::editor::diff_model::{build_diff_lines, is_change, parse_diff_args, DiffState};
        let d = self.inner.delegate();
        let opts = parse_diff_args(&args);
        let rel = d
            .path
            .strip_prefix(&d.root_dir)
            .unwrap_or(&d.path)
            .to_string_lossy()
            .to_string();
        let root = d.root_dir.clone();
        let path = d.path.clone();
        let show_numbers = d.settings.number;
        let base_content = git_file_content(&root, &rel, &opts.base).unwrap_or_default();
        let current = self.editor().buf().content();
        let lines = build_diff_lines(&base_content, &current, &opts);
        let has_changes = lines.iter().any(is_change);
        let base_ref = opts.base.clone();
        if !has_changes {
            let msg = format!("[no changes vs {}]", base_ref);
            self.editor_mut().set_status(msg);
            self.inner.mark_dirty();
            return;
        }
        let ds = DiffState::new(lines, 0, base_ref, opts.context, opts.ignore_ws);
        // Store on delegate for revert access, and emit view for display
        *self.inner.delegate_mut().diff_state_mut() = Some(ds.clone());
        let data: (DiffState, String, PathBuf, bool) = (ds, current, path, show_numbers);
        self.inner.put_command(CM_DIFF_OPEN_VIEW, Some(Box::new(data)));
    }

    fn flush_revert(&mut self) {
        if self.inner.delegate().is_pending_revert() {
            self.inner.delegate_mut().set_pending_revert(false);
            let _ = self.revert_hunk();
        }
    }

    fn flush_nodiff(&mut self) {
        if self.inner.delegate().is_pending_nodiff() {
            self.inner.delegate_mut().set_pending_nodiff(false);
            self.editor_mut().set_status(String::new());
        }
    }

    fn flush_save(&mut self) {
        if !self.inner.delegate().is_save_requested() {
            return;
        }
        self.inner.delegate_mut().set_save_requested(false);
        if self.save_buffer() {
            self.inner.put_broadcast(CM_FS_CHANGED, None);
            let name = self
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let msg = Message::info("editor", format!("Saved: {name}"));
            self.inner
                .put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
    }

    fn flush_sync_settings(&mut self) {
        let cn = self.editor().options().cursor_normal();
        let ci = self.editor().options().cursor_insert();
        let cc = self.editor().options().cursor_command();
        let num = self.editor().options().number();
        let d = self.inner.delegate_mut();
        d.settings.cursor_normal = cn;
        d.settings.cursor_insert = ci;
        d.settings.cursor_command = cc;
        d.settings.number = num;
    }

    fn flush_commands(&mut self) {
        let cmds: Vec<_> = self.inner.delegate_mut().pending_commands_mut().drain(..).collect();
        for (id, data) in cmds {
            self.inner.put_command(id, data);
        }
        let bcasts: Vec<_> = self.inner.delegate_mut().pending_broadcasts_mut().drain(..).collect();
        for (id, data) in bcasts {
            self.inner.put_broadcast(id, data);
        }
        if self.inner.delegate().is_dirty() {
            self.inner.delegate_mut().set_dirty(false);
            self.inner.mark_dirty();
        }
    }
}
