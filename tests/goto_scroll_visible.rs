//! Tests for goto scroll behavior — target line should be visible with margin.

use kairn::views::editor::EditorView;
use txv_core::prelude::*;

fn make_view(lines: usize) -> EditorView {
    let content: String = (1..=lines).map(|i| format!("line{i}\n")).collect();
    let mut view = EditorView::from_text(&content);
    view.set_bounds(Rect::new(0, 0, 40, 15));
    view
}

#[test]
fn goto_scrolls_down_to_show_target() {
    let mut view = make_view(100);
    // Initially at line 0, viewport shows lines 0..14
    view.goto(50, 0);
    let scroll = view.editor().viewport_scroll();
    // Line 50 must be visible: scroll <= 50 < scroll + 15
    assert!(scroll <= 50 && 50 < scroll + 15, "line 50 not visible, scroll={scroll}");
}

#[test]
fn goto_keeps_margin_from_bottom() {
    let mut view = make_view(100);
    view.goto(50, 0);
    let scroll = view.editor().viewport_scroll();
    // Line 50 should NOT be at the very last row — at least 2 lines below it
    assert!(50 + 2 < scroll + 15, "line 50 too close to bottom, scroll={scroll}");
}

#[test]
fn goto_keeps_margin_from_top() {
    let mut view = make_view(100);
    // First scroll down far
    view.goto(90, 0);
    // Then goto line 50 — should scroll up
    view.goto(50, 0);
    let scroll = view.editor().viewport_scroll();
    // Line 50 should NOT be at the very first row — at least 2 lines above it
    assert!(50 >= scroll + 2, "line 50 too close to top, scroll={scroll}");
}

#[test]
fn goto_line_1_at_top_without_margin() {
    let mut view = make_view(100);
    view.goto(80, 0);
    view.goto(0, 0);
    let scroll = view.editor().viewport_scroll();
    // Line 0 is the first line — scroll should be 0 (can't have margin above)
    assert_eq!(scroll, 0);
}

#[test]
fn goto_no_scroll_if_already_visible() {
    let mut view = make_view(100);
    // Put view at scroll=0, cursor at line 5 (well within viewport)
    view.goto(5, 0);
    let scroll = view.editor().viewport_scroll();
    // Line 5 is within 0..14 with margin, no scroll needed
    assert_eq!(scroll, 0, "should not scroll if target already visible with margin");
}

#[test]
fn goto_after_viewport_shrink_rescrolls() {
    let mut view = make_view(100);
    view.goto(50, 0);
    // Shrink viewport
    view.set_bounds(Rect::new(0, 0, 40, 8));
    let scroll = view.editor().viewport_scroll();
    // Line 50 must still be visible in the smaller viewport
    let vh = view.editor().viewport_height();
    assert!(
        scroll <= 50 && 50 < scroll + vh,
        "line 50 not visible after shrink, scroll={scroll} vh={vh}"
    );
}
