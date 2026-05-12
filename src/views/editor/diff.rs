//! Editor diff mode — enter/exit/toggle, delegates to diff_model for computation.

use crate::diff::git_file_content;
use crate::views::editor::diff_model::{build_diff_lines, is_change, parse_diff_args, DiffState};
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

    /// Exit diff mode, restore normal editor.
    pub(super) fn exit_diff(&mut self) {
        self.diff_state = None;
        self.editor.status = String::new();
        self.state.mark_dirty();
    }

    /// Exit diff mode and jump cursor to the buffer line at current diff cursor.
    pub(super) fn exit_diff_at_cursor(&mut self) {
        let buf_line = self.diff_state.as_ref().map(|ds| ds.cursor_buf_line()).unwrap_or(0);
        self.exit_diff();
        self.editor.cursor_line = buf_line;
        self.editor.cursor_col = 0;
        self.ensure_cursor_visible();
    }

    /// Check if currently in diff mode.
    pub(super) fn in_diff_mode(&self) -> bool {
        self.diff_state.is_some()
    }

    fn enter_diff(&mut self, args: &str) {
        let opts = parse_diff_args(args);

        let rel_path = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();

        let base_content = match git_file_content(&self.root_dir, &rel_path, &opts.base) {
            Ok(c) => c,
            Err(e) => {
                self.editor.status = format!("diff: {e}");
                self.state.mark_dirty();
                return;
            }
        };

        let current = self.editor.buffer.content();
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

        self.editor.status = if has_changes {
            format!("[DIFF vs {}]", base_ref)
        } else {
            format!("[no changes vs {}]", base_ref)
        };
        self.state.mark_dirty();
    }
}
