//! Regression: PageUp/PageDown must work in insert mode.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn page_down_moves_cursor_in_insert_mode() {
    let content: String = (1..=80).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    let path = h.state.root_dir().join("big.txt");
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(OpenFileRequest::new(path))));
    h.run_cycles(2);
    // Verify starting at line 1
    assert!(h.contains("Ln 1"), "should start at Ln 1");
    // Enter insert mode
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(1);
    // PageDown should move cursor down by a page
    h.inject_key(KeyCode::PageDown, KeyMod::default());
    h.run_cycles(2);
    // With editor area ~14 lines, cursor should move to around line 14
    assert!(
        h.contains("Ln 14") || h.contains("Ln 13") || h.contains("Ln 15"),
        "cursor should move down by a page in insert mode"
    );
}

#[test]
fn page_up_moves_cursor_in_insert_mode() {
    let content: String = (1..=80).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    let path = h.state.root_dir().join("big.txt");
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(OpenFileRequest::new(path))));
    h.run_cycles(2);
    // Go to line 30 first (in normal mode)
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("30\n");
    h.run_cycles(2);
    assert!(h.contains("Ln 30"), "should be at Ln 30");
    // Enter insert mode
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(1);
    // PageUp should move cursor up by a page
    h.inject_key(KeyCode::PageUp, KeyMod::default());
    h.run_cycles(2);
    // Cursor should have moved up significantly from line 30
    assert!(
        h.contains("Ln 1") || h.contains("Ln 16") || h.contains("Ln 17"),
        "cursor should move up by a page in insert mode"
    );
}
