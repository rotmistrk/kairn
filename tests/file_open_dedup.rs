//! Tests: file open deduplication — no code path may create duplicate tabs.

mod helpers;

use std::path::PathBuf;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE};
use kairn::session;
use kairn::session::schema::{EditorTabState, SessionState, SESSION_VERSION};
use kairn::settings::EditorSettings;
use kairn::slots::SlotId;
use txv_core::event::{KeyCode, KeyMod};

const ALT: KeyMod = KeyMod {
    ctrl: false,
    alt: true,
    shift: false,
};

// ─── CM_OPEN_FILE deduplication ───────────────────────────────────────────────

#[test]
fn open_file_twice_via_command_no_duplicate() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let abs = dir.path().join("src/main.rs");
    let req = OpenFileRequest::new(abs.clone());
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req)));
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1);

    // Open same file again — must NOT create duplicate
    let req2 = OpenFileRequest::new(abs);
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req2)));
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1, "duplicate tab created!");
}

#[test]
fn open_file_with_position_no_duplicate() {
    let dir = temp_project(&[("lib.rs", "line1\nline2\nline3\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let abs = dir.path().join("lib.rs");
    let req = OpenFileRequest::new(abs.clone());
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req)));
    h.run_cycles(2);

    // Open same file at different position — must focus, not duplicate
    let req2 = OpenFileRequest::at(abs, 2, 0);
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req2)));
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1, "position open created duplicate!");
}

// ─── Tree open deduplication ──────────────────────────────────────────────────

#[test]
fn open_from_tree_twice_no_duplicate() {
    let dir = temp_project(&[("only.rs", "hello")]);
    let mut h = TestHarness::new(dir.path());
    // Open from tree
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1);

    // Focus tree, open same file again
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1, "tree open created duplicate!");
}

// ─── Edit command deduplication ───────────────────────────────────────────────

#[test]
fn edit_command_twice_no_duplicate() {
    let dir = temp_project(&[("foo.rs", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // Open via :edit
    h.inject_key(KeyCode::Char('x'), ALT);
    h.run_cycles(1);
    h.inject_str("edit foo.rs");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1);

    // Edit same file again
    h.inject_key(KeyCode::Char('x'), ALT);
    h.run_cycles(1);
    h.inject_str("edit foo.rs");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1, "edit command created duplicate!");
}

// ─── Session restore ──────────────────────────────────────────────────────────

#[test]
fn session_restore_rejects_relative_paths() {
    let dir = temp_project(&[("foo.rs", "x")]);
    let root = dir.path().to_path_buf();

    // Build state with relative path (legacy/corrupt)
    let state = SessionState::builder()
        .version(SESSION_VERSION)
        .layout("auto")
        .active_tab(0)
        .editor_tabs(vec![EditorTabState::new("foo.rs", 0, 0)])
        .build();

    let mut desktop = kairn::build_desktop::build_workspace(&PathBuf::from("."), kairn::settings::GitKeys::default());
    let defaults = EditorSettings::default();
    session::restore_tabs(&mut desktop, &state, &root, &defaults, "base16-eighties.dark");

    // Relative path should be rejected — no tabs opened
    assert_eq!(
        desktop.panel(SlotId::Center as usize).unwrap().tab_count(),
        0,
        "relative path should be rejected by restore"
    );
}

#[test]
fn session_restore_then_open_no_duplicate() {
    let dir = temp_project(&[("bar.rs", "y")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let abs = dir.path().join("bar.rs");
    let abs_str = abs.to_string_lossy().to_string();

    let req = OpenFileRequest::new(abs.clone());
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req)));
    h.run_cycles(2);

    // Broker must know about it
    assert!(h.state.broker_is_open(&abs_str), "broker must track opened file");

    // Opening again must not duplicate
    let req2 = OpenFileRequest::new(abs);
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(req2)));
    h.run_cycles(2);
    assert_eq!(h.state.broker_open_count(), 1, "file duplicated on re-open!");
}
