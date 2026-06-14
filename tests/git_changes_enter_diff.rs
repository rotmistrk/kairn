//! Test: Git changes panel Right-arrow opens diff for modified files.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

const ALT: KeyMod = KeyMod::ALT;

/// Create a git project with an initial commit and a modified file.
fn git_project_with_change() -> tempfile::TempDir {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let repo = git2::Repository::init(dir.path()).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("main.rs")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();
    std::fs::write(dir.path().join("main.rs"), "fn main() {\n    println!(\"hi\");\n}\n").unwrap();
    dir
}

#[test]
fn right_arrow_on_modified_file_opens_diff() {
    let dir = git_project_with_change();
    let mut h = TestHarness::with_size(dir.path(), 200, 50);

    // Run enough cycles for git watcher to detect changes
    h.run_cycles(120);

    // Focus the left panel (F2)
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.run_cycles(2);

    // Switch to Git tab (Alt-2)
    h.inject_key(KeyCode::Char('2'), ALT);
    h.run_cycles(2);

    // Navigate down to the file entry (first is category header)
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(2);

    // Press Right to open with diff and focus editor
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(15);

    // Verify diff view is visible ([diff] in tab title or DIF mode indicator)
    let screen = h.screen_text();
    let has_diff = screen.contains("[HEAD]") || screen.contains("DIF");
    assert!(
        has_diff,
        "Right on modified file should open diff view, screen:\n{}",
        screen
    );
}
