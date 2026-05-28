//! Test: editor diagnostic underline rendering.

use kairn::lsp::diagnostics::{Diagnostic, Severity};
use kairn::views::editor::EditorView;
use txv_core::prelude::*;

#[test]
fn diagnostic_underlines_error_range() {
    let mut view = EditorView::from_text("fn main() {\n    let x = bad;\n}\n");
    view.editor_mut().options_mut().set_number(false);
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view.set_diagnostics(vec![Diagnostic::new(1, 12, 15, Severity::Error, "not found")]);

    view.draw();

    // Check that cells in the diagnostic range have underline + red fg
    let cell = view.buffer().cell(12, 1);
    assert!(cell.style.attrs.underline, "diagnostic range should be underlined");
    assert_eq!(cell.style.fg, Color::Ansi(1), "error should be red");
}

#[test]
fn diagnostic_at_cursor_returns_message() {
    let mut view = EditorView::from_text("line1\nline2\nline3\n");
    view.set_diagnostics(vec![Diagnostic::new(1, 0, 5, Severity::Warning, "unused variable")]);

    // Cursor at line 0 — no diagnostic
    view.editor_mut().set_cursor_line(0);
    assert!(view.diagnostic_at_cursor().is_none());

    // Cursor at line 1 — has diagnostic
    view.editor_mut().set_cursor_line(1);
    assert_eq!(view.diagnostic_at_cursor(), Some("unused variable"));
}

#[test]
fn no_diagnostics_no_crash() {
    let mut view = EditorView::from_text("hello\n");
    view.editor_mut().options_mut().set_number(false);
    view.set_bounds(Rect::new(0, 0, 20, 3));

    // Should not crash with no diagnostics set
    view.draw();
}

#[test]
fn gutter_marker_shows_for_error_line() {
    let mut view = EditorView::from_text("line1\nline2\nline3\n");
    view.editor_mut().options_mut().set_number(true);
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view.set_diagnostics(vec![Diagnostic::new(1, 0, 5, Severity::Error, "err")]);

    view.draw();

    // Gutter marker '●' should appear in the gutter area of line 1
    let gutter_w = view.gutter_width();
    let marker_cell = view.buffer().cell(gutter_w - 1, 1);
    assert_eq!(marker_cell.ch, '●');
}

#[test]
fn gutter_marker_absent_for_clean_line() {
    let mut view = EditorView::from_text("line1\nline2\nline3\n");
    view.editor_mut().options_mut().set_number(true);
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view.set_diagnostics(vec![Diagnostic::new(1, 0, 5, Severity::Error, "err")]);

    view.draw();

    // Line 0 should NOT have a marker
    let gutter_w = view.gutter_width();
    let cell = view.buffer().cell(gutter_w - 1, 0);
    assert_ne!(cell.ch, '●');
}

#[test]
fn clear_diagnostics_removes_markers() {
    let mut view = EditorView::from_text("line1\nline2\n");
    view.editor_mut().options_mut().set_number(true);
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view.set_diagnostics(vec![Diagnostic::new(0, 0, 5, Severity::Error, "err")]);
    view.draw();

    // Marker present
    let gutter_w = view.gutter_width();
    assert_eq!(view.buffer().cell(gutter_w - 1, 0).ch, '●');

    // Clear and redraw
    view.clear_diagnostics();
    view.draw();

    // Marker gone
    assert_ne!(view.buffer().cell(gutter_w - 1, 0).ch, '●');
}

#[test]
fn highest_severity_wins_for_gutter_marker() {
    let mut view = EditorView::from_text("line1\n");
    view.editor_mut().options_mut().set_number(true);
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view.set_diagnostics(vec![
        Diagnostic::new(0, 0, 2, Severity::Warning, "warn"),
        Diagnostic::new(0, 3, 5, Severity::Error, "err"),
    ]);

    view.draw();

    // Error color (red) should win over warning
    let gutter_w = view.gutter_width();
    let cell = view.buffer().cell(gutter_w - 1, 0);
    assert_eq!(cell.ch, '●');
    assert_eq!(cell.style.fg, Color::Ansi(1)); // red = error
}
