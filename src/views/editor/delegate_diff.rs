//! Diff mode operations on KairnDelegate.

use txv_core::prelude::*;

use crate::app_palette::app_palette;
use crate::editor::Editor;
use crate::gutter_signs::compute_gutter_signs;
use crate::lsp::diagnostics::Severity;

use super::delegate::KairnDelegate;

impl KairnDelegate {
    pub(super) fn handle_diff_key(&mut self, key: &txv_core::event::KeyEvent) -> Option<txv_core::view::HandleResult> {
        use txv_core::event::KeyCode;
        use txv_core::view::HandleResult;
        self.diff_state.as_ref()?;
        match key.code() {
            KeyCode::Char('R') if !key.modifiers().ctrl() => {
                self.pending_revert = true;
                self.dirty = true;
                Some(HandleResult::Consumed)
            }
            KeyCode::Char('n') if !key.modifiers().ctrl() => {
                self.diff_next_hunk();
                Some(HandleResult::Consumed)
            }
            KeyCode::Char('N') => {
                self.diff_prev_hunk();
                Some(HandleResult::Consumed)
            }
            KeyCode::Esc => {
                self.diff_state = None;
                self.pending_nodiff = true;
                self.dirty = true;
                Some(HandleResult::Consumed)
            }
            _ => None,
        }
    }

    pub(crate) fn enter_diff_mode(&mut self, editor: &mut Editor, args: &str) {
        use crate::diff::git_file_content;
        use crate::views::editor::diff_model::{build_diff_lines, is_change, parse_diff_args, DiffState};

        let opts = parse_diff_args(args);
        let rel_path = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();
        let base_content = git_file_content(&self.root_dir, &rel_path, &opts.base).unwrap_or_default();
        let current = editor.buf().content();
        let lines = build_diff_lines(&base_content, &current, &opts);
        let has_changes = lines.iter().any(is_change);
        let base_ref = opts.base.clone();
        let status = if has_changes {
            format!("[DIFF vs {}]", base_ref)
        } else {
            format!("[no changes vs {}]", base_ref)
        };
        editor.set_status(status);
        self.diff_state = Some(DiffState::new(lines, 0, base_ref, opts.context, opts.ignore_ws));
        self.dirty = true;
    }

    pub(super) fn diff_next_hunk(&mut self) {
        use super::diff_model::is_change;
        let Some(ds) = &mut self.diff_state else {
            return;
        };
        let start = ds.cursor + 1;
        let pos = ds.lines[start..].iter().position(is_change);
        if let Some(p) = pos {
            ds.cursor = start + p;
        }
    }

    pub(super) fn diff_prev_hunk(&mut self) {
        use super::diff_model::is_change;
        let Some(ds) = &mut self.diff_state else {
            return;
        };
        if ds.cursor == 0 {
            return;
        }
        let pos = ds.lines[..ds.cursor].iter().rposition(is_change);
        if let Some(p) = pos {
            ds.cursor = p;
        }
    }
}
impl KairnDelegate {
    pub(super) fn show_gutter_signs(&self) -> bool {
        self.settings.gutter_signs
    }

    pub(super) fn blame_width(&self) -> u16 {
        if self.blame_state.is_some() {
            24
        } else {
            0
        }
    }

    pub(crate) fn clear_diagnostics(&mut self) {
        if self.diagnostics.is_some() {
            self.diagnostics = None;
            self.dirty = true;
        }
    }

    pub(crate) fn refresh_gutter_signs_from(&mut self, editor: &Editor) {
        if !self.settings.gutter_signs {
            self.gutter_signs.clear();
            return;
        }
        let rel = self
            .path
            .strip_prefix(&self.root_dir)
            .unwrap_or(&self.path)
            .to_string_lossy()
            .to_string();
        let content = editor.buf().content();
        self.gutter_signs = compute_gutter_signs(&self.root_dir, &rel, &content);
        self.dirty = true;
    }

    pub(super) fn diagnostic_severity_at(&self, line: usize) -> Option<Severity> {
        let diags = self.diagnostics.as_ref()?;
        diags
            .iter()
            .filter(|d| d.line == line)
            .map(|d| d.severity)
            .min_by_key(|s| match s {
                Severity::Error => 0,
                Severity::Warning => 1,
                Severity::Info => 2,
                Severity::Hint => 3,
            })
    }
}

pub(super) fn diag_underline_style(severity: Severity) -> Style {
    let app = app_palette();
    match severity {
        Severity::Error => app.diag().error(),
        Severity::Warning => app.diag().warning(),
        Severity::Info => app.diag().info(),
        Severity::Hint => app.diag().hint(),
    }
}

pub(super) fn diag_marker_style(severity: Severity) -> Style {
    let app = app_palette();
    let ps = match severity {
        Severity::Error => app.diag().error(),
        Severity::Warning => app.diag().warning(),
        Severity::Info => app.diag().info(),
        Severity::Hint => app.diag().hint(),
    };
    Style::new(ps.fg(), Style::default().bg())
}
