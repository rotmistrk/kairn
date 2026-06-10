//! KairnEditorDelegate — app-specific extensions for the txv-edit draw engine.

use txv_core::prelude::*;
use txv_edit::editor::EditorAction;
use txv_edit::view::EditorViewDelegate;

use crate::app_palette::app_palette;
use crate::gutter_signs::GutterSign;
use crate::lsp::diagnostics::{Diagnostic, Severity};

/// Kairn's delegate providing git signs, diagnostics overlay, and LSP triggers.
pub(super) struct KairnEditorDelegate<'a> {
    pub(super) gutter_signs: &'a [(usize, GutterSign)],
    pub(super) diagnostics: Option<&'a [Diagnostic]>,
    pub(super) show_gutter_signs: bool,
    pub(super) blame_w: u16,
    pub(super) number: bool,
}

impl EditorViewDelegate for KairnEditorDelegate<'_> {
    fn extra_gutter_width(&self) -> u16 {
        if !self.number {
            return 0;
        }
        let sign_w: u16 = if self.show_gutter_signs {
            1
        } else {
            0
        };
        sign_w + self.blame_w
    }

    fn gutter_sign(&self, line: usize) -> Option<(char, Style)> {
        if !self.show_gutter_signs {
            return None;
        }
        // Git signs only — diagnostic markers rendered by kairn's draw_diagnostics
        self.gutter_signs.iter().find(|(l, _)| *l == line).map(|(_, s)| {
            let app = app_palette();
            match s {
                GutterSign::Added => ('▎', app.diff().added()),
                GutterSign::Modified => ('▎', app.git().modified()),
                GutterSign::Deleted => ('▸', app.diff().deleted()),
            }
        })
    }

    fn extra_style(&self, line: usize, col: usize) -> Option<Style> {
        let diags = self.diagnostics?;
        for d in diags {
            if d.line == line && col >= d.col_start && col < d.col_end {
                return Some(diag_underline_style(d.severity));
            }
        }
        None
    }

    fn line_decorations(&self, _line: usize) -> &[txv_edit::view::delegate::LineDecoration] {
        &[]
    }

    fn on_action(&mut self, _action: &EditorAction) -> bool {
        false
    }

    fn highlight_match_style(&self) -> Style {
        app_palette().editor().highlight_match()
    }

    fn highlight_other_bg(&self) -> Color {
        app_palette().editor().highlight_other().bg()
    }

    fn matchparen_style(&self) -> Style {
        app_palette().editor().matchparen()
    }
}

fn diag_underline_style(severity: Severity) -> Style {
    let app = app_palette();
    match severity {
        Severity::Error => app.diag().error(),
        Severity::Warning => app.diag().warning(),
        Severity::Info => app.diag().info(),
        Severity::Hint => app.diag().hint(),
    }
}

fn diag_marker_style(severity: Severity) -> Style {
    let app = app_palette();
    let ps = match severity {
        Severity::Error => app.diag().error(),
        Severity::Warning => app.diag().warning(),
        Severity::Info => app.diag().info(),
        Severity::Hint => app.diag().hint(),
    };
    Style::new(ps.fg(), Style::default().bg())
}
