//! Test that the sidekick (completion popup) renders at the correct position.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_LSP_COMPLETION;
use kairn::lsp::requests::{CompletionItem, CompletionKind};
use txv_core::event::{Event, KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

/// The completion popup should appear BELOW the cursor line, not at row 0.
#[test]
fn sidekick_popup_appears_below_cursor() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    let x = foo\n}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open file and focus editor
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("main.rs"),
        ))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);

    // Go to end of "foo" on line 2 and enter insert mode
    h.inject_str("2G$");
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('a'), none()); // append after cursor
    h.run_cycles(1);

    // Inject completion items
    h.backend.inject(Event::Command {
        id: CM_LSP_COMPLETION,
        data: Some(Box::new(vec![
            CompletionItem::new("foobar", None, None, CompletionKind::Other),
            CompletionItem::new("foobaz", None, None, CompletionKind::Other),
        ])),
    });
    h.run_cycles(2);

    // The popup should NOT appear on row 0 or row 1 (those are above the cursor).
    // Cursor is on line 2 of the file. With tree panel, the editor content starts
    // at some offset. The popup should be on a row BELOW the cursor row.
    let row0 = h.row(0);
    let row1 = h.row(1);
    assert!(!row0.contains("foobar"), "popup should not be at row 0, got: {}", row0);
    assert!(!row1.contains("foobar"), "popup should not be at row 1, got: {}", row1);

    // The popup SHOULD appear somewhere on screen
    assert!(
        h.content_contains("foobar"),
        "popup 'foobar' should be visible somewhere on screen"
    );

    // Find which row has the popup — it should be row 3 or below
    // (cursor is on file line 2, which renders around screen row 2-3)
    let mut popup_row = None;
    for y in 0..24 {
        if h.row(y).contains("foobar") {
            popup_row = Some(y);
            break;
        }
    }
    let popup_row = popup_row.expect("popup should be on some row");
    assert!(
        popup_row >= 2,
        "popup should be below cursor (row 2+), but was at row {}",
        popup_row
    );
}
