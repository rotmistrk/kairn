//! Editor diff mode — inline diff rendering toggled by :diff / :nodiff / Ctrl-D.

use crate::diff::{git_file_content, DiffOptions};
use crate::views::editor::{DiffTag, EditorView};

impl EditorView {
    /// Toggle diff mode. If already in diff mode, exit. Otherwise compute and enter.
    pub(super) fn toggle_diff(&mut self, args: &str) {
        if self.diff_lines.is_some() && args.is_empty() {
            self.diff_lines = None;
            self.editor.status = String::new();
            self.state.dirty = true;
            return;
        }
        self.enter_diff(args);
    }

    /// Exit diff mode.
    pub(super) fn exit_diff(&mut self) {
        self.diff_lines = None;
        self.editor.status = String::new();
        self.state.dirty = true;
    }

    /// Returns the diff foreground color for a line, or None if not in diff mode.
    pub(super) fn diff_line_color(&self, line_idx: usize) -> Option<txv_core::cell::Color> {
        use txv_core::cell::Color;
        let tags = self.diff_lines.as_ref()?;
        match tags.get(line_idx) {
            Some(DiffTag::Added) => Some(Color::Ansi(2)),
            Some(DiffTag::Removed) => Some(Color::Ansi(1)),
            Some(DiffTag::Context) => Some(Color::Ansi(8)),
            None => None,
        }
    }

    fn enter_diff(&mut self, args: &str) {
        let mut base = "HEAD".to_string();
        for arg in args.split_whitespace() {
            if arg == "-w" {
                // ignore whitespace — handled in diff options
            } else if !arg.starts_with('-') {
                base = arg.to_string();
            }
        }
        self.diff_base = base.clone();

        let rel_path = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();

        let base_content = match git_file_content(&self.root_dir, &rel_path, &base) {
            Ok(c) => c,
            Err(e) => {
                self.editor.status = format!("diff: {e}");
                self.state.dirty = true;
                return;
            }
        };

        let current = self.editor.buffer.content();
        let tags = compute_line_tags(&base_content, &current, args.contains("-w"));
        if tags.iter().all(|t| *t == DiffTag::Context) {
            self.diff_lines = Some(tags);
            self.editor.status = format!("----[no diff vs {base}]----");
        } else {
            self.diff_lines = Some(tags);
            self.editor.status = format!("[diff vs {base}]");
        }
        self.state.dirty = true;
    }
}

/// Compute per-line diff tags between base and current content.
fn compute_line_tags(base: &str, current: &str, ignore_ws: bool) -> Vec<DiffTag> {
    use similar::{ChangeTag, TextDiff};

    let opts = DiffOptions {
        context: usize::MAX,
        ignore_whitespace: ignore_ws,
    };
    let _ = opts; // DiffOptions used for unified_diff, here we use similar directly

    let diff = TextDiff::from_lines(base, current);
    let mut tags = Vec::new();
    for change in diff.iter_all_changes() {
        match change.tag() {
            ChangeTag::Equal => tags.push(DiffTag::Context),
            ChangeTag::Insert => tags.push(DiffTag::Added),
            ChangeTag::Delete => tags.push(DiffTag::Removed),
        }
    }
    tags
}
