//! Editor diff mode — enter/exit/toggle, delegates to diff_model for computation.

use crate::diff::git_file_content;
use crate::views::editor::diff_model::{build_diff_lines, is_change, parse_diff_args, DiffLine, DiffState};
use crate::views::editor::EditorView;

impl EditorView {
    /// Toggle diff mode. If already in diff, exit. Otherwise compute and enter.
    pub fn toggle_diff(&mut self, args: &str) {
        if self.diff_state.is_some() && args.is_empty() {
            self.exit_diff();
            return;
        }
        self.enter_diff(args);
    }

    /// Returns Some(base_content, base_ref) if -y was requested, for the handler to create a split.
    pub fn try_diff_side_by_side(&mut self, args: &str) -> Option<(String, String)> {
        let opts = parse_diff_args(args);
        if !opts.side_by_side {
            return None;
        }
        let rel_path = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();
        match git_file_content(&self.root_dir, &rel_path, &opts.base) {
            Ok(content) => Some((content, opts.base)),
            Err(e) => {
                self.editor.set_status(format!("diff: {e}"));
                self.state.mark_dirty();
                None
            }
        }
    }

    /// Exit diff mode, restore normal editor.
    pub(super) fn exit_diff(&mut self) {
        self.diff_state = None;
        self.editor.set_status(String::new());
        self.state.mark_dirty();
    }

    /// Exit diff mode and jump cursor to the buffer line at current diff cursor.
    pub(super) fn exit_diff_at_cursor(&mut self) {
        let buf_line = self.diff_state.as_ref().map(|ds| ds.cursor_buf_line()).unwrap_or(0);
        self.exit_diff();
        self.editor.set_cursor_line(buf_line);
        self.editor.set_cursor_col(0);
        self.ensure_cursor_visible();
    }

    /// Check if currently in diff mode.
    pub(super) fn in_diff_mode(&self) -> bool {
        self.diff_state.is_some()
    }

    /// Set diff state directly (for testing).
    pub fn set_diff_state(&mut self, state: DiffState) {
        self.diff_state = Some(state);
    }

    /// Set side-by-side diff state.
    pub fn set_sbs_state(&mut self, state: super::sbs_model::SbsDiffState) {
        self.diff_state = None;
        self.sbs_state = Some(state);
        self.state.mark_dirty();
    }

    /// Check if in side-by-side diff mode.
    pub(super) fn in_sbs_mode(&self) -> bool {
        self.sbs_state.is_some()
    }

    /// Revert the hunk under the diff cursor: replace Added lines with Deleted text.
    /// Returns a status message or error.
    pub fn revert_hunk(&mut self) -> Result<String, String> {
        let ds = self.diff_state.as_ref().ok_or("Not in diff mode")?;
        let cursor = ds.cursor;
        if cursor >= ds.lines.len() {
            return Err("Cursor out of range".to_string());
        }
        if !is_change(&ds.lines[cursor]) {
            return Err("Cursor not on a change".to_string());
        }

        let (start, end) = self.find_hunk_bounds(cursor);
        let (buf_lines, deleted_text) = self.collect_hunk_data(start, end);
        self.apply_revert(&buf_lines, &deleted_text, start);
        self.rebuild_diff_after_revert();
        Ok("Hunk reverted".to_string())
    }

    fn find_hunk_bounds(&self, cursor: usize) -> (usize, usize) {
        let ds = self.diff_state.as_ref().unwrap_or_else(|| unreachable!());
        let mut start = cursor;
        while start > 0 && is_change(&ds.lines[start - 1]) {
            start -= 1;
        }
        let mut end = cursor + 1;
        while end < ds.lines.len() && is_change(&ds.lines[end]) {
            end += 1;
        }
        (start, end)
    }

    fn collect_hunk_data(&self, start: usize, end: usize) -> (Vec<usize>, Vec<String>) {
        let ds = self.diff_state.as_ref().unwrap_or_else(|| unreachable!());
        let mut buf_lines: Vec<usize> = Vec::new();
        let mut deleted_text: Vec<String> = Vec::new();
        for line in &ds.lines[start..end] {
            match line {
                DiffLine::Added { buf_line } => buf_lines.push(*buf_line),
                DiffLine::Deleted { text, .. } => deleted_text.push(text.clone()),
                _ => {}
            }
        }
        (buf_lines, deleted_text)
    }

    fn apply_revert(&mut self, buf_lines: &[usize], deleted_text: &[String], start: usize) {
        let mut buf = self.editor.buf();
        buf.begin_group();

        if !buf_lines.is_empty() {
            let first = buf_lines[0];
            let last = buf_lines.last().copied().unwrap_or(first);
            let start_off = buf.line_col_to_offset(first, 0).unwrap_or(0);
            let end_off = if last + 1 < buf.line_count() {
                buf.line_col_to_offset(last + 1, 0).unwrap_or(buf.content().len())
            } else {
                buf.content().len()
            };
            if end_off > start_off {
                buf.delete(start_off, end_off);
            }
            if !deleted_text.is_empty() {
                let insert = deleted_text.join("\n") + "\n";
                let off = buf.line_col_to_offset(first, 0).unwrap_or(buf.content().len());
                buf.insert(off, &insert);
            }
        } else if !deleted_text.is_empty() {
            let insert_line = self.revert_insert_line(start);
            let insert = deleted_text.join("\n") + "\n";
            let off = buf.line_col_to_offset(insert_line, 0).unwrap_or(buf.content().len());
            buf.insert(off, &insert);
        }

        buf.end_group();
    }

    fn revert_insert_line(&self, start: usize) -> usize {
        let ds = self.diff_state.as_ref().unwrap_or_else(|| unreachable!());
        if start > 0 {
            match &ds.lines[start - 1] {
                DiffLine::Context { buf_line, .. } => buf_line + 1,
                DiffLine::Added { buf_line } => buf_line + 1,
                _ => 0,
            }
        } else {
            0
        }
    }

    fn rebuild_diff_after_revert(&mut self) {
        let ds = self.diff_state.as_ref().unwrap_or_else(|| unreachable!());
        let args = if ds.ignore_ws {
            "-w"
        } else {
            ""
        };
        let base_ref = ds.base_ref.clone();
        let context_lines = ds.context_lines;
        self.diff_state = None;
        let full_args = if args.is_empty() {
            format!("-U{context_lines} {base_ref}")
        } else {
            format!("-U{context_lines} {args} {base_ref}")
        };
        self.enter_diff(&full_args);
    }

    fn enter_diff(&mut self, args: &str) {
        let opts = parse_diff_args(args);
        if opts.side_by_side {
            return; // Handled by the view's command handler
        }

        let rel_path = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();

        let base_content = git_file_content(&self.root_dir, &rel_path, &opts.base).unwrap_or_default();

        let current = self.editor.buf().content();
        let lines = build_diff_lines(&base_content, &current, &opts);
        let has_changes = lines.iter().any(is_change);
        let base_ref = opts.base.clone();

        self.diff_state = Some(DiffState {
            lines,
            scroll: 0,
            cursor: 0,
            base_ref: base_ref.clone(),
            context_lines: opts.context,
            ignore_ws: opts.ignore_ws,
        });

        let status_msg = if has_changes {
            format!("[DIFF vs {}]", base_ref)
        } else {
            format!("[no changes vs {}]", base_ref)
        };
        self.editor.set_status(status_msg);
        self.state.mark_dirty();
    }
}
