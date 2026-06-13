//! Tests: dual gutter sign columns (git LEFT, diagnostic RIGHT).

mod helpers;

use kairn::gutter_signs::GutterSign;
use kairn::lsp::diagnostics::{Diagnostic, Severity};
use kairn::views::editor::EditorView;
use txv_core::prelude::*;

fn make_view(text: &str) -> EditorView {
    let mut view = EditorView::from_text(text);
    view.editor_mut().options_mut().set_number(true);
    view.editor_mut().options_mut().set_gutter_signs(true);
    view.set_bounds(Rect::new(0, 0, 40, 5));
    view
}

#[test]
fn git_marker_in_left_column() {
    let mut view = make_view("line1\nline2\nline3\n");
    view.set_gutter_signs_data(vec![(1, GutterSign::Modified)]);
    view.draw();

    let gw = view.gutter_width();
    // Left sign column is at gutter_w - 2 (two sign columns before text)
    let cell = view.buffer().cell(gw - 2, 1);
    assert_eq!(cell.ch(), '▎', "git marker should be in left sign column");
}

#[test]
fn diagnostic_marker_in_right_column() {
    let mut view = make_view("line1\nline2\nline3\n");
    view.set_diagnostics(vec![Diagnostic::new(1, 0, 5, Severity::Error, "err")]);
    view.draw();

    let gw = view.gutter_width();
    // Right sign column is at gutter_w - 1 (rightmost, adjacent to text)
    let cell = view.buffer().cell(gw - 1, 1);
    assert_eq!(cell.ch(), '●', "diagnostic marker should be in right sign column");
}

#[test]
fn both_signs_on_same_line() {
    let mut view = make_view("line1\nline2\nline3\n");
    view.set_gutter_signs_data(vec![(1, GutterSign::Added)]);
    view.set_diagnostics(vec![Diagnostic::new(1, 0, 5, Severity::Warning, "warn")]);
    view.draw();

    let gw = view.gutter_width();
    let left_cell = view.buffer().cell(gw - 2, 1);
    let right_cell = view.buffer().cell(gw - 1, 1);
    assert_eq!(left_cell.ch(), '▎', "git marker in left column");
    assert_eq!(right_cell.ch(), '●', "diagnostic marker in right column");
}

#[test]
fn only_git_marker_right_column_empty() {
    let mut view = make_view("line1\nline2\nline3\n");
    view.set_gutter_signs_data(vec![(1, GutterSign::Deleted)]);
    view.draw();

    let gw = view.gutter_width();
    let left_cell = view.buffer().cell(gw - 2, 1);
    let right_cell = view.buffer().cell(gw - 1, 1);
    assert_eq!(left_cell.ch(), '▸', "git deleted marker in left column");
    assert_eq!(right_cell.ch(), ' ', "right column empty without diagnostics");
}

#[test]
fn only_diagnostic_left_column_empty() {
    let mut view = make_view("line1\nline2\nline3\n");
    view.set_diagnostics(vec![Diagnostic::new(0, 0, 5, Severity::Info, "info")]);
    view.draw();

    let gw = view.gutter_width();
    let left_cell = view.buffer().cell(gw - 2, 0);
    let right_cell = view.buffer().cell(gw - 1, 0);
    assert_eq!(left_cell.ch(), ' ', "left column empty without git sign");
    assert_eq!(right_cell.ch(), '●', "diagnostic in right column");
}
