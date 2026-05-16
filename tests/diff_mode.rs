//! Tests for :diff / :nodiff editor mode.

use kairn::views::editor::EditorView;
use txv_core::event::{KeyCode, KeyMod};
use txv_core::prelude::*;

fn send_ex(view: &mut EditorView, cmd: &str) {
    let colon = Event::Key(txv_core::event::KeyEvent {
        code: KeyCode::Char(':'),
        modifiers: KeyMod::default(),
    });
    view.handle(&colon);
    for ch in cmd.chars() {
        let ev = Event::Key(txv_core::event::KeyEvent {
            code: KeyCode::Char(ch),
            modifiers: KeyMod::default(),
        });
        view.handle(&ev);
    }
    let enter = Event::Key(txv_core::event::KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyMod::default(),
    });
    view.handle(&enter);
}

#[test]
fn diff_does_not_modify_content() {
    let mut view = EditorView::from_text("hello\nworld\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));
    send_ex(&mut view, "diff");
    assert_eq!(view.editor.buf().content(), "hello\nworld\n");
}

#[test]
fn diff_sets_status() {
    let mut view = EditorView::from_text("hello\nworld\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));
    send_ex(&mut view, "diff");
    // No git repo → error status mentioning "diff"
    assert!(
        view.editor.status.contains("diff"),
        "status should mention diff: {:?}",
        view.editor.status
    );
}

#[test]
fn nodiff_clears_diff_mode() {
    let mut view = EditorView::from_text("hello\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));
    // Force diff_lines to simulate being in diff mode
    send_ex(&mut view, "diff");
    send_ex(&mut view, "nodiff");
    assert!(view.editor.status.is_empty());
}

#[test]
fn toggle_diff_via_command() {
    let mut view = EditorView::from_text("hello\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    let sink = EventSink::new();
    view.set_sink(sink.clone());

    // Send CM_DIFF command (simulates Ctrl-D)
    let cmd = Event::Command {
        id: kairn::commands::CM_DIFF,
        data: None,
    };
    let result = view.handle(&cmd);
    assert_eq!(result, HandleResult::Consumed);
    // Status should mention diff (error since no git)
    assert!(
        view.editor.status.contains("diff"),
        "CM_DIFF should trigger diff: {:?}",
        view.editor.status
    );
}
