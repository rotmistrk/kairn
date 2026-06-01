use super::*;
use crate::views::editor::EditorView;
use txv_core::prelude::*;

#[test]
fn wide_char_positions_correct() {
    let mut view = EditorView::from_text("A✅B");
    view.editor.options.number = false;
    view.set_bounds(Rect::new(0, 0, 20, 1));
    view.draw();
    let buf = view.buffer();
    assert_eq!(buf.cell(0, 0).ch, 'A');
    assert_eq!(buf.cell(1, 0).ch, '✅');
    assert_eq!(buf.cell(3, 0).ch, 'B');
}

#[test]
fn matchparen_highlights_matching_bracket() {
    let mut view = EditorView::from_text("foo(bar)");
    view.editor.options.number = false;
    view.editor.options.matchparen = true;
    view.editor.cursor_col = 3;
    view.set_bounds(Rect::new(0, 0, 20, 1));
    view.draw();
    let buf = view.buffer();
    let cell = buf.cell(7, 0);
    assert_eq!(cell.ch, ')');
    assert!(cell.style.attrs.bold, "matching paren should be bold");
}

#[test]
fn rainbow_brackets_colors_by_depth() {
    let result = rainbow_brackets("a(b(c))");
    assert_eq!(result.len(), 4);
    assert_eq!(result[0].0, 1);
    assert_eq!(result[1].0, 3);
    assert_ne!(result[0].1, result[1].1);
    assert_eq!(result[0].1, result[3].1);
}

#[test]
fn rainbow_brackets_depth_carries_across_lines() {
    let (map1, depth) = rainbow_brackets_with_depth("fn main() {", 0);
    assert!(depth > 0);
    let (map2, _) = rainbow_brackets_with_depth("    println!()", depth);
    assert!(!map2.is_empty());
    let inner_color = map2[0].1;
    let outer_color = map1[0].1;
    assert_ne!(inner_color, outer_color);
}

#[test]
fn rainbow_brackets_colored_on_non_cursor_line() {
    let mut view = EditorView::from_text("hello\nfoo(bar)");
    view.editor.options.number = false;
    view.editor.options.rainbow = true;
    view.editor.cursor_line = 0;
    view.set_bounds(Rect::new(0, 0, 20, 2));
    view.draw();
    let buf = view.buffer();
    let cell = buf.cell(3, 1);
    assert_eq!(cell.ch, '(');
    assert_ne!(cell.style.fg, Color::Reset);
}

#[test]
fn indent_guides_drawn_at_tab_stops() {
    let mut view = EditorView::from_text("        hello");
    view.editor.options.number = false;
    view.editor.options.guides = true;
    view.set_bounds(Rect::new(0, 0, 20, 1));
    view.draw();
    let buf = view.buffer();
    assert_eq!(buf.cell(4, 0).ch, '\u{250A}');
}
