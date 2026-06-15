//! EditorView diff mode methods via extension trait.

use std::path::PathBuf;

use txv_core::message::Message;

use super::methods::EditorViewExt;
use super::EditorView;
use crate::commands::{CM_DIFF_OPEN_VIEW, CM_FILE_CLOSED, CM_FS_CHANGED, CM_TAB_CLOSE};

pub trait EditorViewDiffExt {
    fn toggle_diff(&mut self, args: &str);
    fn set_diff_state(&mut self, state: super::diff_model::DiffState);
    fn set_sbs_state(&mut self, state: super::sbs_model::SbsDiffState);
    fn revert_hunk(&mut self) -> Result<String, String>;
    fn flush_pending(&mut self);
}

impl EditorViewDiffExt for EditorView {
    fn toggle_diff(&mut self, args: &str) {
        self.delegate_mut().set_pending_diff(args.to_string());
    }

    fn set_diff_state(&mut self, state: super::diff_model::DiffState) {
        *self.delegate_mut().diff_state_mut() = Some(state);
        let status = self.editor().status().to_string();
        if !status.is_empty() {
            let msg = Message::info("editor", status);
            self.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
        }
        self.mark_dirty();
    }

    fn set_sbs_state(&mut self, _state: super::sbs_model::SbsDiffState) {}

    fn revert_hunk(&mut self) -> Result<String, String> {
        use super::diff_model::is_change;
        let ds = self.delegate().diff_state_ref().as_ref().ok_or("Not in diff mode")?;
        let cursor = ds.cursor;
        if cursor >= ds.lines.len() {
            return Err("Cursor out of range".to_string());
        }
        if !is_change(&ds.lines[cursor]) {
            return Err("Cursor not on a change".to_string());
        }
        let (start, end) = find_hunk_bounds(&ds.lines, cursor);
        let ds = self.delegate().diff_state_ref().as_ref().ok_or("Not in diff mode")?;
        let (buf_lines, deleted_text, insert_line) = collect_hunk_data(ds, start, end);
        apply_revert(self, &buf_lines, &deleted_text, insert_line);
        *self.delegate_mut().diff_state_mut() = None;
        self.mark_dirty();
        Ok("Hunk reverted".to_string())
    }

    fn flush_pending(&mut self) {
        flush_force_close(self);
        flush_diff(self);
        flush_revert(self);
        flush_nodiff(self);
        flush_save(self);
        flush_sync_settings(self);
        flush_commands(self);
    }
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

fn collect_hunk_data(ds: &super::diff_model::DiffState, start: usize, end: usize) -> (Vec<usize>, Vec<String>, usize) {
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

fn apply_revert(view: &mut EditorView, buf_lines: &[usize], deleted_text: &[String], insert_line: usize) {
    let mut buf = view.editor_mut().buf();
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
    view.editor_mut().clamp_cursor();
}

fn flush_force_close(view: &mut EditorView) {
    if !view.delegate().is_force_close() {
        return;
    }
    view.delegate_mut().set_force_close(false);
    view.editor_mut().buf().mark_saved();
    let p = view.path().to_string_lossy().to_string();
    view.put_command(CM_FILE_CLOSED, Some(Box::new(p)));
    view.put_command(CM_TAB_CLOSE, None);
}

fn flush_diff(view: &mut EditorView) {
    if view.delegate().pending_diff_ref().is_none() {
        return;
    }
    let args = view.delegate_mut().take_pending_diff().unwrap_or_default();
    use crate::diff::git_file_content;
    use crate::views::editor::diff_model::{build_diff_lines, is_change, parse_diff_args, DiffState};
    let d = view.delegate();
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
    let current = view.editor().buf().content();
    let lines = build_diff_lines(&base_content, &current, &opts);
    let has_changes = lines.iter().any(is_change);
    let base_ref = opts.base.clone();
    if !has_changes {
        let msg = format!("[no changes vs {}]", base_ref);
        view.editor_mut().set_status(msg);
        view.mark_dirty();
        return;
    }
    let ds = DiffState::new(lines, 0, base_ref, opts.context, opts.ignore_ws);
    *view.delegate_mut().diff_state_mut() = Some(ds.clone());
    let data: (DiffState, String, PathBuf, bool, bool, String) =
        (ds, current, path, show_numbers, opts.side_by_side, base_content);
    view.put_command(CM_DIFF_OPEN_VIEW, Some(Box::new(data)));
}

fn flush_revert(view: &mut EditorView) {
    if view.delegate().is_pending_revert() {
        view.delegate_mut().set_pending_revert(false);
        let _ = view.revert_hunk();
    }
}

fn flush_nodiff(view: &mut EditorView) {
    if view.delegate().is_pending_nodiff() {
        view.delegate_mut().set_pending_nodiff(false);
        view.editor_mut().set_status(String::new());
    }
}

fn flush_save(view: &mut EditorView) {
    if !view.delegate().is_save_requested() {
        return;
    }
    view.delegate_mut().set_save_requested(false);
    if view.save_now() {
        view.put_broadcast(CM_FS_CHANGED, None);
        let name = view
            .path()
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let msg = Message::info("editor", format!("Saved: {name}"));
        view.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
    }
}

fn flush_sync_settings(view: &mut EditorView) {
    let cn = view.editor().options().cursor_normal();
    let ci = view.editor().options().cursor_insert();
    let cc = view.editor().options().cursor_command();
    let num = view.editor().options().number();
    let d = view.delegate_mut();
    d.settings.cursor_normal = cn;
    d.settings.cursor_insert = ci;
    d.settings.cursor_command = cc;
    d.settings.number = num;
}

fn flush_commands(view: &mut EditorView) {
    if view.delegate().is_dirty() {
        view.delegate_mut().set_dirty(false);
        view.mark_dirty();
    }
}
