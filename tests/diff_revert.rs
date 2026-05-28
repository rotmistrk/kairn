//! Tests for diff revert hunk feature (R hotkey / :revert).

use kairn::views::editor::diff_model::{DiffLine, DiffState};
use kairn::views::editor::EditorView;
use txv_core::event::{KeyCode, KeyEvent, KeyMod};
use txv_core::prelude::*;

fn make_diff_state(lines: Vec<DiffLine>, cursor: usize) -> DiffState {
    DiffState::new(lines, cursor, "HEAD", 2, false)
}

#[test]
fn revert_added_lines() {
    let mut view = EditorView::from_text("aaa\nbbb\nccc\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Added { buf_line: 1 },
            DiffLine::Context {
                buf_line: 2,
                base_line: 1,
            },
        ],
        1,
    ));

    let result = view.revert_hunk();
    assert!(result.is_ok(), "revert_hunk failed: {:?}", result);
    assert_eq!(view.editor().buf().content(), "aaa\nccc\n");
}

#[test]
fn revert_deleted_lines() {
    let mut view = EditorView::from_text("aaa\nccc\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Deleted {
                text: "bbb".to_string(),
                base_line: 1,
            },
            DiffLine::Context {
                buf_line: 1,
                base_line: 2,
            },
        ],
        1,
    ));

    let result = view.revert_hunk();
    assert!(result.is_ok(), "revert_hunk failed: {:?}", result);
    assert_eq!(view.editor().buf().content(), "aaa\nbbb\nccc\n");
}

#[test]
fn revert_replaced_lines() {
    let mut view = EditorView::from_text("aaa\nXXX\nccc\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Deleted {
                text: "bbb".to_string(),
                base_line: 1,
            },
            DiffLine::Added { buf_line: 1 },
            DiffLine::Context {
                buf_line: 2,
                base_line: 2,
            },
        ],
        1,
    ));

    let result = view.revert_hunk();
    assert!(result.is_ok(), "revert_hunk failed: {:?}", result);
    assert_eq!(view.editor().buf().content(), "aaa\nbbb\nccc\n");
}

#[test]
fn revert_on_context_line_errors() {
    let mut view = EditorView::from_text("aaa\nbbb\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Context {
                buf_line: 1,
                base_line: 1,
            },
        ],
        0,
    ));

    let result = view.revert_hunk();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not on a change"));
}

#[test]
fn revert_not_in_diff_mode_errors() {
    let mut view = EditorView::from_text("aaa\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    let result = view.revert_hunk();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Not in diff mode"));
}

#[test]
fn revert_via_r_hotkey() {
    let mut view = EditorView::from_text("aaa\nbbb\nccc\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    let sink = EventSink::new();
    view.set_sink(sink.clone());

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Added { buf_line: 1 },
            DiffLine::Context {
                buf_line: 2,
                base_line: 1,
            },
        ],
        1,
    ));

    let key = Event::Key(KeyEvent {
        code: KeyCode::Char('R'),
        modifiers: KeyMod::default(),
    });
    view.handle(&key);

    assert_eq!(view.editor().buf().content(), "aaa\nccc\n");
}

#[test]
fn revert_multi_line_added() {
    let mut view = EditorView::from_text("aaa\nb1\nb2\nb3\nccc\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Added { buf_line: 1 },
            DiffLine::Added { buf_line: 2 },
            DiffLine::Added { buf_line: 3 },
            DiffLine::Context {
                buf_line: 4,
                base_line: 1,
            },
        ],
        2, // middle of hunk
    ));

    let result = view.revert_hunk();
    assert!(result.is_ok());
    assert_eq!(view.editor().buf().content(), "aaa\nccc\n");
}

#[test]
fn revert_multi_line_replacement() {
    let mut view = EditorView::from_text("aaa\nX1\nX2\nccc\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    view.set_diff_state(make_diff_state(
        vec![
            DiffLine::Context {
                buf_line: 0,
                base_line: 0,
            },
            DiffLine::Deleted {
                text: "old1".to_string(),
                base_line: 1,
            },
            DiffLine::Deleted {
                text: "old2".to_string(),
                base_line: 2,
            },
            DiffLine::Deleted {
                text: "old3".to_string(),
                base_line: 3,
            },
            DiffLine::Added { buf_line: 1 },
            DiffLine::Added { buf_line: 2 },
            DiffLine::Context {
                buf_line: 3,
                base_line: 4,
            },
        ],
        4, // on an Added line
    ));

    let result = view.revert_hunk();
    assert!(result.is_ok());
    assert_eq!(view.editor().buf().content(), "aaa\nold1\nold2\nold3\nccc\n");
}
