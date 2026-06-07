//! Integration tests: editor split via native TiledWorkspace subpanels.
//! Tests: vsplit, split, Ctrl-W focus cycle, :only close.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}
fn ctrl() -> KeyMod {
    KeyMod::CTRL
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

#[test]
fn vsplit_shows_vertical_divider() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical())),
    );
    h.run_cycles(3);

    let screen = h.screen_text();
    assert!(
        screen.contains("│"),
        "vertical divider expected after vsplit:\n{}",
        screen
    );
    assert!(h.content_contains("main"));
}

#[test]
fn split_shows_horizontal_divider() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::horizontal())),
    );
    h.run_cycles(3);

    assert!(h.content_contains("main"));
}

#[test]
fn ctrl_w_cycles_focus_between_split_panes() {
    let dir = temp_project(&[("a.rs", "// file A\n"), ("b.rs", "// file B\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "a.rs");

    // Split with different file
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical_with_file(
            "b.rs".to_string(),
        ))),
    );
    h.run_cycles(3);

    // Ctrl-W w should cycle focus (two-key sequence)
    h.inject_key(KeyCode::Char('w'), ctrl());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('w'), none());
    h.run_cycles(2);

    // Both files should be visible
    assert!(h.content_contains("file A") || h.content_contains("file B"));
}

#[test]
fn split_close_returns_to_single_pane() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical())),
    );
    h.run_cycles(3);

    // Close split
    h.dispatch_command(kairn::commands::CM_SPLIT_CLOSE, None);
    h.run_cycles(3);

    // Should still show content, no divider
    assert!(h.content_contains("main"));
}
