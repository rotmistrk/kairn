mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn tree_shows_files_on_start() {
    let dir = temp_project(&[("hello.rs", ""), ("world.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    assert!(h.contains("hello.rs"));
    assert!(h.contains("world.rs"));
}

#[test]
fn tree_down_moves_cursor() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("b.rs"));
}

#[test]
fn tree_dirs_sort_before_files() {
    let dir = temp_project(&[("z.rs", ""), ("adir/x.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let screen = h.screen_text();
    let dir_pos = screen.find("adir").unwrap_or(usize::MAX);
    let file_pos = screen.find("z.rs").unwrap_or(0);
    assert!(dir_pos < file_pos, "dirs should sort before files");
}

#[test]
fn tree_enter_on_dir_expands() {
    let dir = temp_project(&[("sub/inner.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    // First item should be "sub" dir
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("inner.rs"));
}
