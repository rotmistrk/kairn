//! Scenario tests for multi-root workspace: file tree, add/remove root.

use std::path::PathBuf;

use kairn::views::tree::FileTreeView;
use kairn::workspace_roots::WorkspaceRoots;
use txv_core::prelude::*;
use txv_widgets::file_tree::FileTreeData;
use txv_widgets::tree_view::TreeData;

#[test]
fn file_tree_data_multi_root_shows_root_nodes() {
    let tmp = tempfile::tempdir().unwrap();
    let root_a = tmp.path().join("alpha");
    let root_b = tmp.path().join("beta");
    std::fs::create_dir_all(root_a.join("src")).unwrap();
    std::fs::write(root_a.join("src/main.rs"), "fn main() {}").unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    std::fs::write(root_b.join("readme.md"), "# Beta").unwrap();

    let data = FileTreeData::with_roots(vec![root_a.clone(), root_b.clone()]);

    // Two top-level root nodes visible
    assert!(data.visible_count() >= 2, "expected at least 2 visible nodes");
    assert!(data.label(0).contains("alpha"), "first root should be alpha");
    assert!(data.label(1).contains("beta"), "second root should be beta");
    assert!(data.is_expandable(0), "root nodes are expandable");
}

#[test]
fn file_tree_view_multi_root_navigates() {
    let tmp = tempfile::tempdir().unwrap();
    let root_a = tmp.path().join("proj_a");
    let root_b = tmp.path().join("proj_b");
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::write(root_a.join("a.txt"), "aaa").unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    std::fs::write(root_b.join("b.txt"), "bbb").unwrap();

    let sink = EventSink::new();
    let mut view = FileTreeView::with_roots(vec![root_a, root_b], None);
    view.set_bounds(Rect::new(0, 0, 40, 10));
    view.set_sink(sink.clone());

    // Navigate down to second root
    let down = Event::Key(KeyEvent::new(KeyCode::Down, KeyMod::default()));
    view.handle(&down);
    // No crash, navigation works
}

#[test]
fn workspace_roots_add_remove() {
    let mut roots = WorkspaceRoots::new(PathBuf::from("/home/user/proj"));

    assert_eq!(roots.len(), 1);
    assert!(roots.add(PathBuf::from("/home/user/lib")));
    assert_eq!(roots.len(), 2);

    // Duplicate add returns false
    assert!(!roots.add(PathBuf::from("/home/user/lib")));
    assert_eq!(roots.len(), 2);

    // root_for finds correct root
    let r = roots.root_for(std::path::Path::new("/home/user/lib/src/main.rs"));
    assert_eq!(r.path(), PathBuf::from("/home/user/lib").as_path());

    // Remove works
    assert!(roots.remove(std::path::Path::new("/home/user/lib")));
    assert_eq!(roots.len(), 1);

    // Cannot remove last root
    assert!(!roots.remove(std::path::Path::new("/home/user/proj")));
    assert_eq!(roots.len(), 1);
}

#[test]
fn workspace_roots_color_assignment() {
    let mut roots = WorkspaceRoots::new(PathBuf::from("/a"));
    roots.add(PathBuf::from("/b"));
    roots.add(PathBuf::from("/c"));

    // Each root gets a distinct color
    let colors: Vec<_> = roots.all().iter().map(|r| r.color()).collect();
    assert_ne!(colors[0], colors[1]);
    assert_ne!(colors[1], colors[2]);
}

#[test]
fn file_tree_badge_color_on_root_nodes() {
    let tmp = tempfile::tempdir().unwrap();
    let root_a = tmp.path().join("alpha");
    let root_b = tmp.path().join("beta");
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();

    let mut data = FileTreeData::with_roots(vec![root_a, root_b]);
    let green = txv_core::cell::Color::Ansi(2);
    let yellow = txv_core::cell::Color::Ansi(3);
    data.set_root_badge_colors(vec![green, yellow]);

    // Root nodes should have badge colors
    assert_eq!(data.badge_color(0), Some(green));
    assert_eq!(data.badge_color(1), Some(yellow));
}

#[test]
fn file_tree_badge_color_none_in_single_root() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("file.txt"), "hi").unwrap();

    let data = FileTreeData::new(tmp.path());
    // Single root: no badges
    for i in 0..data.visible_count() {
        let id = data.visible_id(i);
        assert_eq!(data.badge_color(id), None);
    }
}
