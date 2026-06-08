//! KairnEditorDelegate — app-specific extensions for the txv-edit draw engine.

use txv_core::prelude::*;
use txv_edit::editor::{Editor, EditorAction};
use txv_edit::view::draw::compute_gutter_width;
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

    fn draw_gutter_sign(&self, buf: &mut Buffer, line: usize, x: u16, y: u16) {
        if !self.show_gutter_signs {
            return;
        }
        if let Some(sign) = self.gutter_signs.iter().find(|(l, _)| *l == line).map(|(_, s)| *s) {
            let app = app_palette();
            let (ch, style) = match sign {
                GutterSign::Added => ('▎', app.diff().added()),
                GutterSign::Modified => ('▎', app.git().modified()),
                GutterSign::Deleted => ('▸', app.diff().deleted()),
            };
            buf.put(x, y, ch, style);
        }
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

    fn post_draw(&self, buf: &mut Buffer, editor: &Editor) {
        // Diagnostic gutter markers at end of line-number area
        let Some(diags) = self.diagnostics else {
            return;
        };
        let scroll = editor.viewport_scroll();
        let h = buf.height() as usize;
        for d in diags {
            if d.line >= scroll && d.line < scroll + h {
                let y = (d.line - scroll) as u16;
                let marker_style = diag_marker_style(d.severity);
                // Place marker just before the text area starts
                let gutter_w = compute_gutter_width(editor, self);
                if gutter_w > 0 {
                    buf.put(gutter_w - 1, y, '●', marker_style);
                }
            }
        }
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
