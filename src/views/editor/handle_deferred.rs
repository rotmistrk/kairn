//! Deferred action processing: diff, revert, save, force close.

use std::fs::metadata;
use std::path::PathBuf;

use txv_core::message::Message;

use super::delegate::KairnDelegate;
use crate::commands::{CM_DIFF_OPEN_VIEW, CM_FILE_CLOSED, CM_FS_CHANGED, CM_TAB_CLOSE};
use crate::editor::Editor;
use crate::views::editor::diff_model::{is_change, DiffLine, DiffState};

impl KairnDelegate {
    pub(crate) fn process_deferred(&mut self, editor: &mut Editor) {
        self.flush_force_close(editor);
        self.flush_diff(editor);
        self.flush_revert(editor);
        self.flush_nodiff(editor);
        self.flush_save(editor);
    }

    fn flush_force_close(&mut self, editor: &mut Editor) {
        if !self.force_close {
            return;
        }
        self.force_close = false;
        editor.buf().mark_saved();
        let p = self.path.to_string_lossy().to_string();
        self.emit(CM_FILE_CLOSED, Some(Box::new(p)));
        self.emit(CM_TAB_CLOSE, None);
    }

    fn flush_diff(&mut self, editor: &mut Editor) {
        if self.pending_diff.is_none() {
            return;
        }
        let args = self.pending_diff.take().unwrap_or_default();
        use crate::diff::git_file_content;
        use crate::views::editor::diff_model::{build_diff_lines, parse_diff_args};
        let opts = parse_diff_args(&args);
        let rel = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();
        let base_content = git_file_content(&self.root_dir, &rel, &opts.base).unwrap_or_default();
        let current = editor.buf().content();
        let lines = build_diff_lines(&base_content, &current, &opts);
        let has_changes = lines.iter().any(is_change);
        let base_ref = opts.base.clone();
        if !has_changes {
            editor.set_status(format!("[no changes vs {}]", base_ref));
            self.dirty = true;
            return;
        }
        let ds = DiffState::new(lines, 0, base_ref, opts.context, opts.ignore_ws);
        self.diff_state = Some(ds.clone());
        let path = self.path.clone();
        let data: (DiffState, String, PathBuf, bool, bool, String) =
            (ds, current, path, self.settings.number, opts.side_by_side, base_content);
        self.emit(CM_DIFF_OPEN_VIEW, Some(Box::new(data)));
    }

    fn flush_revert(&mut self, editor: &mut Editor) {
        if !self.pending_revert {
            return;
        }
        self.pending_revert = false;
        let Some(ds) = &self.diff_state else {
            return;
        };
        let cursor = ds.cursor;
        if cursor >= ds.lines.len() || !is_change(&ds.lines[cursor]) {
            return;
        }
        let (buf_lines, deleted_text, insert_line) = Self::collect_hunk(&ds.lines, cursor);
        Self::apply_hunk(editor, &buf_lines, &deleted_text, insert_line);
        self.diff_state = None;
        self.dirty = true;
    }

    fn collect_hunk(lines: &[DiffLine], cursor: usize) -> (Vec<usize>, Vec<String>, usize) {
        let mut s = cursor;
        while s > 0 && is_change(&lines[s - 1]) {
            s -= 1;
        }
        let mut e = cursor + 1;
        while e < lines.len() && is_change(&lines[e]) {
            e += 1;
        }
        let mut buf_lines: Vec<usize> = Vec::new();
        let mut deleted_text: Vec<String> = Vec::new();
        for line in &lines[s..e] {
            match line {
                DiffLine::Added { buf_line } => buf_lines.push(*buf_line),
                DiffLine::Deleted { text, .. } => deleted_text.push(text.clone()),
                _ => {}
            }
        }
        let insert_line = if buf_lines.is_empty() && s > 0 {
            match &lines[s - 1] {
                DiffLine::Context { buf_line, .. } | DiffLine::Added { buf_line } => buf_line + 1,
                _ => 0,
            }
        } else {
            0
        };
        (buf_lines, deleted_text, insert_line)
    }

    fn apply_hunk(editor: &mut Editor, buf_lines: &[usize], deleted_text: &[String], insert_line: usize) {
        let mut buf = editor.buf();
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
        editor.clamp_cursor();
    }

    fn flush_nodiff(&mut self, editor: &mut Editor) {
        if !self.pending_nodiff {
            return;
        }
        self.pending_nodiff = false;
        editor.set_status(String::new());
    }

    fn flush_save(&mut self, editor: &mut Editor) {
        if !self.save_requested {
            return;
        }
        self.save_requested = false;
        let content = editor.buf().content();
        if self.store.save(&content).is_ok() {
            editor.buf().mark_saved();
            self.disk_mtime = metadata(&self.path).and_then(|m| m.modified()).ok();
            self.emit_broadcast(CM_FS_CHANGED, None);
            let name = self.path.file_name().unwrap_or_default().to_string_lossy().to_string();
            let msg = Message::info("editor", format!("Saved: {name}"));
            self.emit(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            self.refresh_gutter_signs_from(editor);
        }
    }
}
