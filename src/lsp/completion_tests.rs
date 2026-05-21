//! Unit tests for CompletionPopup: navigation, boundary clamping, scrolling.

use super::completion::*;
use super::requests::{CompletionItem, CompletionKind};
use txv_core::prelude::*;

fn items(labels: &[&str]) -> Vec<CompletionItem> {
    labels
        .iter()
        .map(|l| CompletionItem {
            label: l.to_string(),
            detail: None,
            insert_text: None,
            kind: CompletionKind::Other,
        })
        .collect()
}

#[test]
fn show_and_navigate() {
    let mut popup = CompletionPopup::new();
    popup.show(items(&["foo", "bar", "baz"]), 5, 10);
    assert!(popup.visible);
    assert_eq!(popup.selected, 0);
    assert_eq!(popup.selected_text(), Some("foo"));

    popup.next();
    assert_eq!(popup.selected_text(), Some("bar"));

    popup.prev();
    assert_eq!(popup.selected_text(), Some("foo"));

    popup.prev(); // wraps
    assert_eq!(popup.selected_text(), Some("baz"));
}

#[test]
fn show_empty_hides() {
    let mut popup = CompletionPopup::new();
    popup.show(Vec::new(), 0, 0);
    assert!(!popup.visible);
}

#[test]
fn draw_renders_items() {
    let mut popup = CompletionPopup::new();
    popup.show(items(&["hello", "world"]), 0, 0);
    let mut buf = Buffer::new(20, 5);
    popup.draw(&mut buf);
    let cell = buf.cell(1, 1);
    assert_eq!(cell.ch, 'h');
}

#[test]
fn draw_clamps_right_edge() {
    let mut popup = CompletionPopup::new();
    popup.show(items(&["longitem", "another"]), 18, 0);
    let mut buf = Buffer::new(20, 10);
    popup.draw(&mut buf);
    // Popup should shift left so it fits within 20 cols
    let mut found = false;
    for x in 0..20 {
        if buf.cell(x, 1).ch == 'l' {
            found = true;
            break;
        }
    }
    assert!(found, "popup should be visible within buffer width");
}

#[test]
fn draw_above_when_no_room_below() {
    let mut popup = CompletionPopup::new();
    popup.show(items(&["aaa", "bbb", "ccc"]), 0, 4);
    // Buffer is 6 rows. Anchor at row 4 → only 1 row below. Should show above.
    let mut buf = Buffer::new(20, 6);
    popup.draw(&mut buf);
    // Items should appear above anchor (rows 1-3)
    let cell = buf.cell(1, 1);
    assert_eq!(cell.ch, 'a');
}

#[test]
fn scroll_down_past_visible_window() {
    let mut popup = CompletionPopup::new();
    let labels: Vec<&str> = (0..12)
        .map(|i| match i {
            0 => "item0",
            1 => "item1",
            2 => "item2",
            3 => "item3",
            4 => "item4",
            5 => "item5",
            6 => "item6",
            7 => "item7",
            8 => "item8",
            9 => "item9",
            10 => "itemA",
            _ => "itemB",
        })
        .collect();
    popup.show(items(&labels), 0, 0);
    for _ in 0..9 {
        popup.next();
    }
    assert_eq!(popup.selected, 9);
    assert_eq!(popup.scroll, 2); // selected(9) - page(8) + 1 = 2

    let mut buf = Buffer::new(20, 12);
    popup.draw(&mut buf);
    // First visible item should be "item2" (scroll=2)
    let cell = buf.cell(1, 1);
    assert_eq!(cell.ch, 'i');
}

#[test]
fn scroll_up_wraps_to_end() {
    let mut popup = CompletionPopup::new();
    let labels: Vec<&str> = (0..12)
        .map(|i| match i {
            0 => "item0",
            1 => "item1",
            2 => "item2",
            3 => "item3",
            4 => "item4",
            5 => "item5",
            6 => "item6",
            7 => "item7",
            8 => "item8",
            9 => "item9",
            10 => "itemA",
            _ => "itemB",
        })
        .collect();
    popup.show(items(&labels), 0, 0);
    popup.prev(); // wraps to last item (11)
    assert_eq!(popup.selected, 11);
    assert_eq!(popup.scroll, 4); // 11 - 8 + 1 = 4
}
