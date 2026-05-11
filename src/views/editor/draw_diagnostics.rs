//! Diagnostic underline rendering for the editor.

use txv_core::prelude::*;

use crate::lsp::diagnostics::{Diagnostic, Severity};

use super::EditorView;

impl EditorView {
    /// Overlay diagnostic underlines on the surface for visible lines.
    pub(super) fn draw_diagnostics(&self, surface: &mut Surface) {
        let diagnostics = match &self.diagnostics {
            Some(diags) => diags,
            None => return,
        };
        let b = self.state.bounds;
        let gutter_w = self.gutter_width();
        let scroll = self.editor.viewport_scroll;
        let visible_lines = b.h as usize;

        for diag in diagnostics {
            if diag.line < scroll || diag.line >= scroll + visible_lines {
                continue;
            }
            let row = (diag.line - scroll) as u16;
            let y = b.y + row;
            let text_x = b.x + gutter_w;
            let style = diag_style(diag.severity);

            let start = text_x + diag.col_start as u16;
            let end = text_x + diag.col_end as u16;
            let max_x = b.x + b.w;

            for x in start..end.min(max_x) {
                let cell = surface.cell(x, y);
                let merged = Style {
                    fg: style.fg,
                    bg: cell.style.bg,
                    attrs: Attrs {
                        underline: true,
                        ..cell.style.attrs
                    },
                };
                surface.put(x, y, cell.ch, merged);
            }
        }
    }

    /// Get the diagnostic message at the current cursor line (for status bar).
    pub fn diagnostic_at_cursor(&self) -> Option<&str> {
        let diags = self.diagnostics.as_ref()?;
        let line = self.editor.cursor_line;
        diags.iter().find(|d| d.line == line).map(|d| d.message.as_str())
    }

    /// Set diagnostics for this editor view.
    pub fn set_diagnostics(&mut self, diagnostics: Vec<Diagnostic>) {
        self.diagnostics = Some(diagnostics);
        self.state.dirty = true;
    }

    /// Clear diagnostics.
    pub fn clear_diagnostics(&mut self) {
        self.diagnostics = None;
    }
}

fn diag_style(severity: Severity) -> Style {
    let fg = match severity {
        Severity::Error => Color::Ansi(1),
        Severity::Warning => Color::Ansi(3),
        Severity::Info => Color::Ansi(6),
        Severity::Hint => Color::Ansi(8),
    };
    Style {
        fg,
        attrs: Attrs {
            underline: true,
            ..Attrs::default()
        },
        ..Style::default()
    }
}
