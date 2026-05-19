//! Integration tests: blame mode toggle.

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
fn blame_command_does_not_crash_without_git() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    // Trigger blame via command
    h.dispatch_command(kairn::commands::CM_BLAME, None);
    h.run_cycles(5);

    // Editor should still be functional
    assert!(h.content_contains("main"));
}

#[test]
fn blame_toggle_on_git_repo() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    // Initialize a git repo so blame has something to work with
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir.path())
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output()
        .ok();
    std::process::Command::new("git")
        .args(["commit", "-m", "init", "--author", "Test <t@t.com>"])
        .current_dir(dir.path())
        .env("GIT_COMMITTER_NAME", "Test")
        .env("GIT_COMMITTER_EMAIL", "t@t.com")
        .output()
        .ok();

    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    // Toggle blame on
    h.dispatch_command(kairn::commands::CM_BLAME, None);
    h.run_cycles(10);

    // Editor still shows content
    assert!(h.content_contains("main"));

    // Toggle blame off
    h.dispatch_command(kairn::commands::CM_BLAME, None);
    h.run_cycles(5);

    assert!(h.content_contains("main"));
}

#[test]
fn blame_on_new_unsaved_file_no_crash() {
    let dir = temp_project(&[("new.rs", "// brand new\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "new.rs");

    h.dispatch_command(kairn::commands::CM_BLAME, None);
    h.run_cycles(5);

    assert!(h.content_contains("brand new"));
}
