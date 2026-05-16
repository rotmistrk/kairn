//! Test: editor diagnostic underline rendering.

use kairn::lsp::diagnostics::{Diagnostic, Severity};
use kairn::views::editor::EditorView;
use txv_core::prelude::*;

#[test]
fn diagnostic_underlines_error_range() {
    let mut view = EditorView::from_text("fn main() {\n    let x = bad;\n}\n");
    view.editor.options.number = false;
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view.set_diagnostics(vec![Diagnostic {
        line: 1,
        col_start: 12,
        col_end: 15,
        severity: Severity::Error,
        message: "not found".into(),
    }]);

    view.draw();

    // Check that cells in the diagnostic range have underline + red fg
    let cell = view.buffer().cell(12, 1);
    assert!(cell.style.attrs.underline, "diagnostic range should be underlined");
    assert_eq!(cell.style.fg, Color::Ansi(1), "error should be red");
}

#[test]
fn diagnostic_at_cursor_returns_message() {
    let mut view = EditorView::from_text("line1\nline2\nline3\n");
    view.set_diagnostics(vec![Diagnostic {
        line: 1,
        col_start: 0,
        col_end: 5,
        severity: Severity::Warning,
        message: "unused variable".into(),
    }]);

    // Cursor at line 0 — no diagnostic
    view.editor.cursor_line = 0;
    assert!(view.diagnostic_at_cursor().is_none());

    // Cursor at line 1 — has diagnostic
    view.editor.cursor_line = 1;
    assert_eq!(view.diagnostic_at_cursor(), Some("unused variable"));
}

#[test]
fn no_diagnostics_no_crash() {
    let mut view = EditorView::from_text("hello\n");
    view.editor.options.number = false;
    view.set_bounds(Rect::new(0, 0, 20, 3));

    // Should not crash with no diagnostics set
    view.draw();
}
