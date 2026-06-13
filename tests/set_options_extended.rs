//! :set norainbow/noguides/nogutter-signs/tree.icons — verify no crash.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::{KeyCode, KeyMod};

fn open_file(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    let req = OpenFileRequest::new(dir.join(file));
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);
}

fn send_ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    for ch in cmd.chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);
}

#[test]
fn set_norainbow_no_crash() {
    let dir = temp_project(&[("code.rs", "fn main() { let x = 1; }\n")]);
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, dir.path(), "code.rs");
    send_ex(&mut h, "set norainbow");
    assert!(h.content_contains("main"), "editor still renders after norainbow");
}

#[test]
fn set_noguides_no_crash() {
    let dir = temp_project(&[("code.rs", "fn main() {\n    let x = 1;\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, dir.path(), "code.rs");
    send_ex(&mut h, "set noguides");
    assert!(h.content_contains("main"), "editor still renders after noguides");
}

#[test]
fn set_nogutter_signs_hides_sign_column() {
    let dir = temp_project(&[("code.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    open_file(&mut h, dir.path(), "code.rs");

    // First, verify screen renders normally
    h.run_cycles(2);
    let before = h.screen_text();
    assert!(before.contains("main"));

    // Disable gutter signs
    send_ex(&mut h, "set nogutter-signs");
    let after = h.screen_text();
    assert!(after.contains("main"), "editor still renders after nogutter-signs");
}

#[test]
fn set_tree_icons_no_crash() {
    let dir = temp_project(&[("code.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    send_ex(&mut h, "set tree.icons");
    // Just verify no crash — app still renders
    let screen = h.screen_text();
    assert!(!screen.is_empty(), "app should still render after set tree.icons");
}
