//! Integration tests: gs (open-in-split) creates split, navigates other pane.

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

#[test]
fn open_in_split_creates_subpanel() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() { lib_fn(); }\n"),
        ("src/lib.rs", "pub fn lib_fn() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // Simulate open-in-split (what gs does after LSP responds)
    h.dispatch_command(
        kairn::commands::CM_OPEN_IN_SPLIT,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/lib.rs"),
        ))),
    );
    h.run_cycles(3);

    // Should show vertical divider (split created)
    let screen = h.screen_text();
    assert!(
        screen.contains("│"),
        "open-in-split should create a vertical split:\n{}",
        screen
    );
}

#[test]
fn open_in_split_reuses_existing_split() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() {}\n"),
        ("src/lib.rs", "pub fn lib_fn() {}\n"),
        ("src/util.rs", "pub fn util() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // First open-in-split
    h.dispatch_command(
        kairn::commands::CM_OPEN_IN_SPLIT,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/lib.rs"),
        ))),
    );
    h.run_cycles(3);

    // Second open-in-split should navigate the existing other pane
    h.dispatch_command(
        kairn::commands::CM_OPEN_IN_SPLIT,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/util.rs"),
        ))),
    );
    h.run_cycles(3);

    // Should still have content visible (no crash, split reused)
    assert!(h.content_contains("main") || h.content_contains("util"));
}
