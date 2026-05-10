// === :e path edge cases ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn colon_e_subdirectory_file() {
    let dir = temp_project(&[("start.txt", "starter"), ("sub/deep.txt", "DEEP_CONTENT")]);
    let mut h = TestHarness::new(dir.path());
    // First open start.txt: navigate tree past sub/ dir to start.txt
    // Tree order: sub/ (dir first), start.txt
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Verify we have an editor open
    assert!(h.contains("starter"), "should have start.txt open");
    // Now :e sub/deep.txt
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e sub/deep.txt\n");
    h.run_cycles(1);
    assert!(h.contains("DEEP_CONTENT"), "expected subdirectory file content");
}

#[test]
fn colon_e_nonexistent_creates_empty_buffer() {
    let dir = temp_project(&[("hello.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e newfile.txt\n");
    h.run_cycles(1);
    // Should not crash, should show empty buffer (tilde lines)
    assert!(h.contains("~"), "expected tilde for empty buffer");
}
