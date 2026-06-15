//! Tests for :diff / :nodiff editor mode.

use kairn::views::editor::{EditorView, EditorViewDiffExt, EditorViewExt};
use txv_core::event::{KeyCode, KeyMod};
use txv_core::prelude::*;

fn send_ex(view: &mut EditorView, cmd: &str) {
    let colon = Event::Key(txv_core::event::KeyEvent::new(KeyCode::Char(':'), KeyMod::default()));
    view.handle(&colon);
    for ch in cmd.chars() {
        let ev = Event::Key(txv_core::event::KeyEvent::new(KeyCode::Char(ch), KeyMod::default()));
        view.handle(&ev);
    }
    let enter = Event::Key(txv_core::event::KeyEvent::new(KeyCode::Enter, KeyMod::default()));
    view.handle(&enter);
    // Process deferred actions (pending_diff, etc.)
    view.handle(&Event::Tick);
}

#[test]
fn diff_does_not_modify_content() {
    let mut view = kairn::views::editor::build::from_text("hello\nworld\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));
    send_ex(&mut view, "diff");
    assert_eq!(view.editor().buf().content(), "hello\nworld\n");
}

#[test]
fn diff_sets_status() {
    let mut view = kairn::views::editor::build::from_text("");
    view.set_bounds(Rect::new(0, 0, 80, 24));
    send_ex(&mut view, "diff");
    // Empty content vs empty base (no git) → no changes → status set
    assert!(
        view.editor().status().contains("no changes"),
        "status should mention no changes: {:?}",
        view.editor().status()
    );
}

#[test]
fn nodiff_clears_diff_mode() {
    let mut view = kairn::views::editor::build::from_text("hello\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));
    // Force diff_lines to simulate being in diff mode
    send_ex(&mut view, "diff");
    send_ex(&mut view, "nodiff");
    assert!(view.editor().status().is_empty());
}

#[test]
fn toggle_diff_via_command() {
    let mut view = kairn::views::editor::build::from_text("hello\n");
    view.set_bounds(Rect::new(0, 0, 80, 24));

    let sink = EventSink::new();
    view.set_sink(sink.clone());

    // Send CM_DIFF command (simulates Ctrl-D)
    let cmd = Event::Command {
        broadcast: false,
        id: kairn::commands::CM_DIFF,
        data: None,
    };
    let result = view.handle(&cmd);
    assert_eq!(result, HandleResult::Consumed);
    // Status should mention DIFF (enters diff mode against empty base)
    assert!(
        view.editor().status().contains("DIFF"),
        "CM_DIFF should trigger diff: {:?}",
        view.editor().status()
    );
}
