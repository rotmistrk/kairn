//! Diagnostic underline rendering for the editor.

use txv_core::prelude::*;

use crate::app_palette::app_palette;
use crate::lsp::diagnostics::{Diagnostic, Severity};

use super::EditorView;

struct DiagOverlay {
    x: u16,
    y: u16,
    ch: char,
    style: Style,
}

impl EditorView {
    /// Overlay diagnostic underlines on the buffer for visible lines.
    pub(super) fn draw_diagnostics(&mut self) {
        let diagnostics = match self.diagnostics.take() {
            Some(diags) => diags,
            None => return,
        };
        let w = self.state.buffer_mut().width();
        let gutter_w = self.gutter_width();
        let scroll = self.editor.viewport_scroll;
        let visible_lines = self.state.buffer_mut().height() as usize;
        let h_off = self.editor.h_scroll;

        let overlays = self.collect_diag_overlays(&diagnostics, w, gutter_w, scroll, visible_lines, h_off);
        for ov in overlays {
            self.state.buffer_mut().put(ov.x, ov.y, ov.ch, ov.style);
        }
        self.diagnostics = Some(diagnostics);
    }

    #[allow(clippy::too_many_arguments)]
    #[rustfmt::skip]
    fn collect_diag_overlays(
        &mut self,
        diagnostics: &[Diagnostic],
        w: u16,
        gutter_w: u16,
        scroll: usize,
        visible_lines: usize,
        h_off: usize,
    ) -> Vec<DiagOverlay> {
        let mut overlays = Vec::new();
        for diag in diagnostics {
            if diag.line < scroll || diag.line >= scroll + visible_lines { continue; }
            let y = (diag.line - scroll) as u16;
            let style = diag_style(diag.severity);
            let col_s = diag.col_start.saturating_sub(h_off);
            let col_e = diag.col_end.saturating_sub(h_off);
            if col_s == col_e || diag.col_end <= h_off { continue; }
            let start = gutter_w + col_s as u16;
            let end = gutter_w + col_e as u16;
            for x in start..end.min(w) {
                let cell = self.state.buffer_mut().cell(x, y);
                let merged = Style {
                    fg: style.fg, bg: cell.style.bg,
                    attrs: Attrs { underline: true, ..cell.style.attrs },
                };
                overlays.push(DiagOverlay { x, y, ch: cell.ch, style: merged });
            }
        }
        overlays
    }

    /// Get the diagnostic message at the current cursor line (for status bar).
    pub fn diagnostic_at_cursor(&self) -> Option<&str> {
        let diags = self.diagnostics.as_ref()?;
        let line = self.editor.cursor_line;
        diags.iter().find(|d| d.line == line).map(|d| d.message.as_str())
    }

    /// Get the highest severity diagnostic at a given line.
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

    /// Set diagnostics for this editor view.
    pub fn set_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics = Some(diagnostics);
        self.state.mark_dirty();
    }

    /// Clear diagnostics (called on buffer edits — LSP will resend after didChange).
    pub fn clear_diagnostics(&mut self) {
        if self.diagnostics.is_some() {
            self.diagnostics = None;
            self.state.mark_dirty();
        }
    }
}

fn diag_style(severity: Severity) -> Style {
    let app = app_palette();
    match severity {
        Severity::Error => app.diag().error(),
        Severity::Warning => app.diag().warning(),
        Severity::Info => app.diag().info(),
        Severity::Hint => app.diag().hint(),
    }
}

/// Style for gutter diagnostic marker (fg only, no bg override).
pub(super) fn diag_marker_style(severity: Severity) -> Style {
    let app = app_palette();
    let ps = match severity {
        Severity::Error => app.diag().error(),
        Severity::Warning => app.diag().warning(),
        Severity::Info => app.diag().info(),
        Severity::Hint => app.diag().hint(),
    };
    Style {
        fg: ps.fg,
        ..Style::default()
    }
}
