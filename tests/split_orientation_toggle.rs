//! Regression test for c46028d: When already split and a file arg is provided,
//! :split/:vsplit should open the file in the other pane, not just toggle orientation.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_and_focus(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

/// When already split, `:vsplit other.rs` should open other.rs in the other pane.
#[test]
fn split_with_file_opens_in_other_pane_when_already_split() {
    let dir = temp_project(&[
        ("main.rs", "fn main() {}\n"),
        ("lib.rs", "pub fn lib_fn() {}\n"),
        ("other.rs", "pub fn other_fn() {}\n"),
    ]);
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    // First split with lib.rs
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical_with_file(
            "lib.rs".to_string(),
        ))),
    );
    h.run_cycles(3);

    // Now already split — issue split with other.rs
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical_with_file(
            "other.rs".to_string(),
        ))),
    );
    h.run_cycles(3);

    // other.rs content should be visible (opened in the other pane)
    assert!(
        h.content_contains("other_fn"),
        "split with file arg when already split should open file in other pane"
    );
}

/// When already split and NO file arg, should toggle orientation (no crash).
#[test]
fn split_no_file_toggles_orientation_when_already_split() {
    let dir = temp_project(&[
        ("main.rs", "fn main() {}\n"),
        ("lib.rs", "pub fn lib_fn() {}\n"),
    ]);
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    // Create initial vertical split
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical_with_file(
            "lib.rs".to_string(),
        ))),
    );
    h.run_cycles(3);

    // Toggle to horizontal (no file arg)
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::horizontal())),
    );
    h.run_cycles(3);

    // Should still show content from both files (orientation changed, not crashed)
    assert!(h.content_contains("main") || h.content_contains("lib_fn"));
}
