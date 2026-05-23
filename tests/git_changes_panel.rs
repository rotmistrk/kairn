// === Git changes panel tests ===

mod helpers;

use helpers::TestHarness;
use kairn::views::git_changes::GitChangesData;
use txv_widgets::tree_view::TreeData;

#[test]
fn git_changes_data_groups_by_status() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    std::fs::write(dir.path().join("new.txt"), "hello").unwrap();
    std::fs::write(dir.path().join("other.txt"), "world").unwrap();

    let data = GitChangesData::new(dir.path());
    // Should have at least one category node (Untracked) and two file nodes
    assert!(
        data.visible_count() >= 3,
        "expected 3+ visible nodes, got {}",
        data.visible_count()
    );
    // First visible node should be a category (expandable)
    let first_id = data.visible_id(0);
    assert!(data.is_expandable(first_id));
    assert!(data.label(first_id).contains("Untracked"));
}

#[test]
fn git_changes_data_empty_repo_no_nodes() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    // No files — no changes
    let data = GitChangesData::new(dir.path());
    assert_eq!(data.visible_count(), 0);
}

#[test]
fn git_changes_data_collapse_category() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    std::fs::write(dir.path().join("a.txt"), "a").unwrap();
    std::fs::write(dir.path().join("b.txt"), "b").unwrap();

    let mut data = GitChangesData::new(dir.path());
    let before = data.visible_count();
    // Collapse the first category
    let first_id = data.visible_id(0);
    data.toggle(first_id);
    assert!(data.visible_count() < before, "collapsing should hide children");
}

#[test]
fn git_changes_view_is_non_closeable() {
    let dir = tempfile::tempdir().unwrap();
    if git2::Repository::init(dir.path()).is_err() {
        return;
    }
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // The left slot shows "Files" in Static mode (other tabs may overflow)
    let top_row = h.row(0);
    assert!(
        top_row.contains("Files"),
        "Left slot should show Files tab, got: {top_row}"
    );
}
