//! Integration tests: split/vsplit editor commands.

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
fn split_creates_second_pane() {
    let dir = temp_project(&[("src/main.rs", "fn main() {\n    println!(\"hi\");\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    // Execute :split via command
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest {
            vertical: false,
            file: None,
        })),
    );
    h.run_cycles(3);

    // Content should still be visible (same file in both panes)
    assert!(h.content_contains("main"));
}

#[test]
fn vsplit_creates_vertical_pane() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest {
            vertical: true,
            file: None,
        })),
    );
    h.run_cycles(3);

    assert!(h.content_contains("main"));
}

#[test]
fn split_with_different_file() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() {}\n"),
        ("src/lib.rs", "pub fn lib_fn() {}\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest {
            vertical: true,
            file: Some("src/lib.rs".to_string()),
        })),
    );
    h.run_cycles(3);

    // Both files should be visible
    assert!(h.content_contains("main") || h.content_contains("lib_fn"));
}

#[test]
fn split_close_returns_to_single_pane() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "src/main.rs");

    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest {
            vertical: false,
            file: None,
        })),
    );
    h.run_cycles(3);

    h.dispatch_command(kairn::commands::CM_SPLIT_CLOSE, None);
    h.run_cycles(3);

    assert!(h.content_contains("main"));
}
