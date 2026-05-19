//! Integration tests: file tree refresh on external filesystem changes.
//! Three layers:
//!   1. File creation — new files appear in tree
//!   2. File deletion/rename — removed/renamed files update in tree
//!   3. Directory operations — new/removed dirs update in tree

mod helpers;

use helpers::{temp_project, TestHarness};

// ═══════════════════════════════════════════════════════════════════════
// Layer 1: File creation — new files must appear after refresh
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn tree_shows_file_created_externally() {
    let dir = temp_project(&[("existing.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    assert!(h.contains("existing.rs"));
    assert!(!h.contains("new_file.rs"));

    // External process creates a file
    std::fs::write(dir.path().join("new_file.rs"), "fn main() {}").unwrap();

    // Simulate enough ticks for periodic refresh (60 ticks)
    h.run_cycles(65);
    assert!(
        h.contains("new_file.rs"),
        "Tree must show externally created file after periodic refresh"
    );
}

#[test]
fn tree_shows_multiple_files_created_externally() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    std::fs::write(dir.path().join("b.rs"), "").unwrap();
    std::fs::write(dir.path().join("c.rs"), "").unwrap();

    h.run_cycles(65);
    assert!(h.contains("b.rs"), "Tree must show b.rs");
    assert!(h.contains("c.rs"), "Tree must show c.rs");
}

// ═══════════════════════════════════════════════════════════════════════
// Layer 2: File deletion and rename — tree must update
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn tree_removes_file_deleted_externally() {
    let dir = temp_project(&[("keep.rs", ""), ("remove_me.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    assert!(h.contains("remove_me.rs"));

    std::fs::remove_file(dir.path().join("remove_me.rs")).unwrap();

    h.run_cycles(65);
    assert!(
        !h.contains("remove_me.rs"),
        "Tree must not show deleted file after refresh"
    );
    assert!(h.contains("keep.rs"));
}

#[test]
fn tree_shows_renamed_file() {
    let dir = temp_project(&[("old_name.rs", "content")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    assert!(h.contains("old_name.rs"));

    std::fs::rename(dir.path().join("old_name.rs"), dir.path().join("new_name.rs")).unwrap();

    h.run_cycles(65);
    assert!(!h.contains("old_name.rs"), "Old name must disappear after rename");
    assert!(h.contains("new_name.rs"), "New name must appear after rename");
}

// ═══════════════════════════════════════════════════════════════════════
// Layer 3: Directory operations — new/removed dirs update in tree
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn tree_shows_directory_created_externally() {
    let dir = temp_project(&[("file.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    assert!(!h.contains("newdir"));

    std::fs::create_dir(dir.path().join("newdir")).unwrap();
    std::fs::write(dir.path().join("newdir/inner.rs"), "").unwrap();

    h.run_cycles(65);
    assert!(h.contains("newdir"), "Tree must show externally created directory");
}

#[test]
fn tree_removes_directory_deleted_externally() {
    let dir = temp_project(&[("mydir/child.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    assert!(h.contains("mydir"));

    std::fs::remove_dir_all(dir.path().join("mydir")).unwrap();

    h.run_cycles(65);
    assert!(
        !h.contains("mydir"),
        "Tree must not show deleted directory after refresh"
    );
}

#[test]
fn tree_shows_file_moved_into_subdir() {
    let dir = temp_project(&[("top.rs", ""), ("sub/placeholder", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    assert!(h.contains("top.rs"));

    std::fs::rename(dir.path().join("top.rs"), dir.path().join("sub/top.rs")).unwrap();

    h.run_cycles(65);
    // top.rs should no longer be at root level
    // (it may still appear if sub/ is expanded, but not at root)
    let screen = h.screen_text();
    // Find "top.rs" — it should only appear indented under sub/ if at all
    let lines: Vec<&str> = screen.lines().collect();
    let root_level_top = lines.iter().any(|l| {
        let trimmed = l.trim_start();
        trimmed.starts_with("top.rs") && l.len() - trimmed.len() < 4
    });
    assert!(
        !root_level_top,
        "top.rs must not appear at root level after being moved to sub/"
    );
}
