//! Regression tests: tab limit enforcement and diff-from-git-changes.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

/// Verify that the center panel never exceeds max_tabs (default 10).
#[test]
fn tab_count_never_exceeds_max() {
    // Create 15 files
    let files: Vec<(String, String)> = (0..15)
        .map(|i| (format!("file{i:02}.rs"), format!("fn f{i}() {{}}")))
        .collect();
    let file_refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let dir = temp_project(&file_refs);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    // Open files from tree: focus tree, navigate, open, return to tree
    for _ in 0..15 {
        h.inject_key(KeyCode::F(2), KeyMod::default()); // focus tree
        h.run_cycles(1);
        h.inject_key(KeyCode::Char('j'), KeyMod::default()); // next file
        h.run_cycles(1);
        h.inject_key(KeyCode::Enter, KeyMod::default()); // open
        h.run_cycles(2);
    }
    h.run_cycles(2);
    // Check tab bar (row 1 of screen, which is the center tab bar)
    // Count distinct "file" occurrences in the tab bar row
    let tab_row = h.row(1);
    let tab_count = (0..15).filter(|i| tab_row.contains(&format!("file{i:02}"))).count();
    assert!(
        tab_count <= 10,
        "Tab bar shows {tab_count} tabs (max 10)!\nTab row: {tab_row}"
    );
}

/// Verify that session restore respects tab limit.
#[test]
fn session_restore_respects_tab_limit() {
    use kairn::session::schema::SessionState;
    use std::fs;
    // Create a project with a saved session of 15 tabs
    let files: Vec<(String, String)> = (0..15)
        .map(|i| (format!("f{i:02}.rs"), format!("// file {i}")))
        .collect();
    let file_refs: Vec<(&str, &str)> = files.iter().map(|(a, b)| (a.as_str(), b.as_str())).collect();
    let dir = temp_project(&file_refs);
    // Write a fake session file with 15 tabs
    let tabs_json: Vec<String> = (0..15)
        .map(|i| {
            let p = dir.path().join(format!("f{i:02}.rs"));
            format!(r#"{{"path":"{}","line":0,"col":0}}"#, p.display())
        })
        .collect();
    let session_json = format!(r#"{{"editor_tabs":[{}],"active_tab":0}}"#, tabs_json.join(","));
    fs::write(dir.path().join(".kairn.state"), &session_json).unwrap();
    // Load and verify
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    // With max_tabs=10, we should not have all 15 open
    let screen = h.screen_text();
    let has_00 = screen.contains("f00");
    let has_14 = screen.contains("f14");
    assert!(!(has_00 && has_14), "Session restored more than max tabs!\n{screen}");
}

/// From git changes tree, Enter on a tracked file must show diff indicators.
#[test]
fn git_changes_enter_opens_diff_not_plain_file() {
    use kairn::commands::{OpenFileRequest, CM_OPEN_FILE};
    use std::process::Command;
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    Command::new("git").args(["init"]).current_dir(root).output().unwrap();
    Command::new("git")
        .args(["config", "user.email", "t@t.com"])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "T"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(root.join("hello.rs"), "fn main() {}\n").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(root)
        .output()
        .unwrap();
    std::fs::write(root.join("hello.rs"), "fn main() { changed }\n").unwrap();

    let mut h = TestHarness::new(root);
    h.run_cycles(3);
    // Directly dispatch what Git Changes would emit: CM_OPEN_FILE with diff=true
    let path = root.join("hello.rs");
    let req = OpenFileRequest::with_diff(path);
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req)));
    h.run_cycles(4);
    // The editor should be in diff mode
    let screen = h.screen_text();
    let has_diff = screen.contains("DIF") || screen.contains("[DIFF") || screen.contains("[HEAD");
    assert!(
        has_diff,
        "OpenFileRequest with diff=true should activate diff mode!\n{screen}"
    );
}
