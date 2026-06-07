//! Regression: cursor must return to editor after M-x is dismissed.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

const ALT: KeyMod = KeyMod::ALT;

/// After M-x is opened and dismissed with Esc, the hardware cursor
/// must NOT remain in the status bar.
#[test]
fn cursor_returns_to_editor_after_mx_dismiss() {
    let dir = temp_project(&[("f.txt", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open file and focus editor
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("f.txt"),
        ))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);

    // Enter insert mode so we get a hardware cursor
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(2);

    let cursor_before = h.backend.cursor();
    assert!(cursor_before.is_some(), "should have cursor in insert mode");
    let before_y = cursor_before.unwrap().y();

    // Open M-x
    h.inject_key(KeyCode::Char('x'), ALT);
    h.run_cycles(2);

    // Dismiss with Esc
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    let cursor_after = h.backend.cursor();
    assert!(cursor_after.is_some(), "should still have cursor after M-x dismiss");
    let after_y = cursor_after.unwrap().y();

    // Cursor should be back in the editor area, not the status bar (last row)
    let height = h.backend.buffer().map(|b| b.height()).unwrap_or(25);
    assert!(
        after_y < height.saturating_sub(1) as u16,
        "cursor y={after_y} should not be in status bar (height={height})"
    );
    assert_eq!(before_y, after_y, "cursor should return to same row");
}
