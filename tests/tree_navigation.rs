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

#[test]
fn tree_collapse_clears_child_rows() {
    let dir = temp_project(&[("sub/a.rs", ""), ("sub/b.rs", ""), ("top.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // Expand "sub" directory
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains("a.rs"));
    assert!(h.contains("b.rs"));

    // Collapse "sub" directory (Left collapses expanded nodes)
    h.inject_key(KeyCode::Left, KeyMod::default());
    h.run_cycles(1);

    // Child items must not appear on screen (no stale rows)
    assert!(!h.content_contains("a.rs"), "a.rs should be gone after collapse");
    assert!(!h.content_contains("b.rs"), "b.rs should be gone after collapse");
}

#[test]
fn tree_collapse_preserves_child_expanded_state() {
    let dir = temp_project(&[("alpha/beta/c.txt", "x"), ("alpha/d.txt", "y")]);
    let mut h = TestHarness::new(dir.path());
    // Focus tree panel
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);
    // Expand 'alpha'
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.contains("beta"), "alpha expanded, beta visible");
    // Move to 'beta' and expand
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(
        h.contains("c.txt"),
        "beta expanded, c.txt visible:\n{}",
        h.screen_text()
    );
    // Go to top (alpha) and collapse
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.inject_key(KeyCode::Left, KeyMod::default());
    h.run_cycles(2);
    assert!(!h.contains("c.txt"), "alpha collapsed, c.txt hidden");
    // Re-expand alpha — beta should still be expanded showing c.txt
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(
        h.contains("c.txt"),
        "beta still expanded after parent re-expand:\n{}",
        h.screen_text()
    );
}
