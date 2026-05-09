use super::*;

#[test]
fn insert_at_start() {
    let mut pt = PieceTable::from_text("hello");
    pt.insert(0, "say ");
    assert_eq!(pt.content(), "say hello");
}

#[test]
fn insert_at_end() {
    let mut pt = PieceTable::from_text("hello");
    pt.insert(5, " world");
    assert_eq!(pt.content(), "hello world");
}

#[test]
fn insert_in_middle() {
    let mut pt = PieceTable::from_text("helo");
    pt.insert(3, "l");
    assert_eq!(pt.content(), "hello");
}

#[test]
fn delete_range() {
    let mut pt = PieceTable::from_text("hello world");
    pt.delete(5, 11);
    assert_eq!(pt.content(), "hello");
}

#[test]
fn undo_redo() {
    let mut pt = PieceTable::from_text("hello");
    pt.insert(5, " world");
    assert_eq!(pt.content(), "hello world");
    pt.undo();
    assert_eq!(pt.content(), "hello");
    pt.redo();
    assert_eq!(pt.content(), "hello world");
}

#[test]
fn line_count() {
    let pt = PieceTable::from_text("line1\nline2\nline3");
    assert_eq!(pt.line_count(), 3);
}

#[test]
fn get_line() {
    let pt = PieceTable::from_text("line1\nline2\nline3");
    assert_eq!(pt.line(0), Some("line1".to_string()));
    assert_eq!(pt.line(1), Some("line2".to_string()));
    assert_eq!(pt.line(2), Some("line3".to_string()));
}

#[test]
fn empty_buffer() {
    let pt = PieceTable::new();
    assert_eq!(pt.len(), 0);
    assert_eq!(pt.line_count(), 1);
}
