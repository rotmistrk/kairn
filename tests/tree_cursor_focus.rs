//! Test: tree cursor style changes based on focus state.

use txv_core::prelude::*;
use txv_widgets::{FileTreeData, TreeView};

fn make_tree() -> TreeView<FileTreeData> {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("a.rs"), "fn main() {}").unwrap();
    std::fs::write(tmp.path().join("b.rs"), "fn test() {}").unwrap();
    let data = FileTreeData::new(tmp.path().to_path_buf());
    let mut tv = TreeView::new(data);
    tv.set_bounds(Rect::new(0, 0, 30, 10));
    // Keep tmp alive by leaking (test only)
    std::mem::forget(tmp);
    tv
}

#[test]
fn focused_cursor_has_blue_bg() {
    let mut tv = make_tree();
    tv.select(); // focused = true
    tv.draw();

    // Cursor row (row 0) should have bg = Ansi(4)
    let cell = tv.buffer().cell(0, 0);
    assert_eq!(
        cell.style().bg(),
        Color::Ansi(4),
        "focused cursor should have blue bg, got {:?}",
        cell.style().bg()
    );
}

#[test]
fn unfocused_cursor_has_subtle_bg() {
    let mut tv = make_tree();
    tv.unselect(); // focused = false
    tv.draw();

    // Cursor row (row 0) should have bg = Ansi(8) (subtle gray)
    let cell = tv.buffer().cell(0, 0);
    assert_eq!(
        cell.style().bg(),
        Color::Ansi(8),
        "unfocused cursor should have gray bg, got {:?}",
        cell.style().bg()
    );
}

#[test]
fn focused_cursor_has_underline() {
    let mut tv = make_tree();
    tv.select();
    tv.draw();

    let cell = tv.buffer().cell(0, 0);
    assert!(
        cell.style().attrs().underline_val(),
        "focused cursor should be underlined"
    );
}

#[test]
fn unfocused_cursor_no_underline() {
    let mut tv = make_tree();
    tv.unselect();
    tv.draw();

    let cell = tv.buffer().cell(0, 0);
    assert!(
        !cell.style().attrs().underline_val(),
        "unfocused cursor should not be underlined"
    );
}
