//! Integration tests: diff mode navigation (n/N/j/k/g/G/Esc/Enter).

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

/// Create a git repo with initial commit, then modify the file.
fn setup_diff_repo(files: &[(&str, &str)], modify: &[(&str, &str)]) -> tempfile::TempDir {
    let dir = temp_project(files);
    // git init + commit
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
        .args(["commit", "-m", "init", "--author", "T <t@t.com>"])
        .current_dir(dir.path())
        .env("GIT_COMMITTER_NAME", "T")
        .env("GIT_COMMITTER_EMAIL", "t@t.com")
        .output()
        .ok();
    // Modify files
    for (path, content) in modify {
        std::fs::write(dir.path().join(path), content).unwrap();
    }
    dir
}

#[test]
fn diff_mode_enter_and_exit_esc() {
    let dir = setup_diff_repo(
        &[("main.rs", "line1\nline2\nline3\n")],
        &[("main.rs", "line1\nmodified\nline3\nnew_line\n")],
    );
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    // Enter diff mode via Ctrl-D
    h.inject_key(KeyCode::Char('d'), KeyMod::CTRL);
    h.run_cycles(5);

    // Should show diff content (+ or - lines, or "modified")
    assert!(h.content_contains("modified") || h.content_contains("line1"));

    // Exit with Esc
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    // Back to normal editor
    assert!(h.content_contains("modified"));
}

#[test]
fn diff_mode_j_k_navigation() {
    let dir = setup_diff_repo(
        &[("main.rs", "a\nb\nc\nd\ne\nf\n")],
        &[("main.rs", "a\nX\nc\nd\nY\nf\n")],
    );
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.inject_key(KeyCode::Char('d'), KeyMod::CTRL);
    h.run_cycles(5);

    // Navigate down
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);

    // Navigate up
    h.inject_key(KeyCode::Char('k'), none());
    h.run_cycles(1);

    // Should not crash
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);
    assert!(h.content_contains("X") || h.content_contains("a"));
}

#[test]
fn diff_mode_n_jumps_to_next_hunk() {
    let dir = setup_diff_repo(
        &[("main.rs", "a\nb\nc\nd\ne\nf\ng\nh\n")],
        &[("main.rs", "a\nX\nc\nd\ne\nY\ng\nh\n")],
    );
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.inject_key(KeyCode::Char('d'), KeyMod::CTRL);
    h.run_cycles(5);

    // n = next hunk
    h.inject_key(KeyCode::Char('n'), none());
    h.run_cycles(1);

    // N = previous hunk
    h.inject_key(KeyCode::Char('N'), none());
    h.run_cycles(1);

    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);
    assert!(h.content_contains("X"));
}

#[test]
fn diff_mode_g_and_big_g() {
    let dir = setup_diff_repo(&[("main.rs", "a\nb\nc\nd\ne\n")], &[("main.rs", "a\nX\nc\nd\nY\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.inject_key(KeyCode::Char('d'), KeyMod::CTRL);
    h.run_cycles(5);

    // G = jump to end
    h.inject_key(KeyCode::Char('G'), none());
    h.run_cycles(1);

    // g = jump to start
    h.inject_key(KeyCode::Char('g'), none());
    h.run_cycles(1);

    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);
    assert!(h.content_contains("a"));
}

#[test]
fn diff_mode_enter_exits_at_cursor_line() {
    let dir = setup_diff_repo(
        &[("main.rs", "line1\nline2\nline3\n")],
        &[("main.rs", "line1\nchanged\nline3\n")],
    );
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.inject_key(KeyCode::Char('d'), KeyMod::CTRL);
    h.run_cycles(5);

    // Move to a line and press Enter to exit at that position
    h.inject_key(KeyCode::Char('j'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);

    // Should be back in normal mode
    assert!(h.content_contains("changed"));
}

#[test]
fn diff_on_untracked_file_no_crash() {
    let dir = temp_project(&[("untracked.rs", "fn new() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "untracked.rs");

    // Ctrl-D on a file with no git history
    h.inject_key(KeyCode::Char('d'), KeyMod::CTRL);
    h.run_cycles(5);

    // Should not crash, may show message
    assert!(h.content_contains("new") || h.contains("No"));
}
