//! Tests: VIS and CMD mode indicators appear in status bar.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

fn open_and_focus(h: &mut TestHarness, path: &std::path::Path) {
    let req = OpenFileRequest::new(path.to_path_buf());
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);
}

#[test]
fn visual_mode_shows_vis_indicator() {
    let dir = temp_project(&[("t.txt", "hello world\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, &dir.path().join("t.txt"));

    // Enter visual mode
    h.inject_key(KeyCode::Char('v'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.contains("VIS"), "status bar should show VIS in visual mode");

    // Exit visual mode
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    assert!(h.contains("NOR"), "status bar should show NOR after Esc");
}

#[test]
fn command_mode_shows_cmd_indicator() {
    let dir = temp_project(&[("t.txt", "hello world\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, &dir.path().join("t.txt"));

    // Enter command mode
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.contains("CMD"), "status bar should show CMD in command mode");

    // Exit command mode
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    assert!(h.contains("NOR"), "status bar should show NOR after Esc");
}
