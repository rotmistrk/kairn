//! Regression test for d21eb1c: Right arrow on a folded todo node with children
//! should expand it, not open notes.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn focus_todo(h: &mut TestHarness) {
    use kairn::handler::downcast_desktop;
    use kairn::slots::{focus_tab_by_title, SlotId};
    let desktop = h.program.desktop_mut();
    if let Some(d) = downcast_desktop(desktop) {
        focus_tab_by_title(d, SlotId::Left, "Todo");
        d.focus_panel(SlotId::Left as usize);
    }
    h.run_cycles(2);
}

fn todo_json(items: &str) -> String {
    format!(r#"{{"version":"2.0","title":"Todo","items":[{items}]}}"#)
}

/// A parent item with children, folded.
fn parent_with_child() -> String {
    r#"{"title":"Parent","completed":"Open","important":false,"folded":true,"items":[{"title":"Child","completed":"Open","important":false,"folded":false,"items":[]}]}"#.to_string()
}

/// Right arrow on a folded node with children should expand (show children),
/// NOT open the notes panel.
#[test]
fn right_arrow_expands_folded_node_with_children() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&parent_with_child());
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    focus_todo(&mut h);

    // Initially folded — "Child" should not be visible
    assert!(h.content_contains("Parent"));
    assert!(!h.content_contains("Child"), "Child should be hidden when folded");

    // Press Right to expand
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(2);

    // After expand, "Child" should be visible
    assert!(h.content_contains("Child"), "Right arrow should expand folded node");
}

/// Right arrow on an expanded leaf node (no children) should open notes.
#[test]
fn right_arrow_on_leaf_opens_notes() {
    let dir = temp_project(&[("x.txt", "")]);
    let item = r#"{"title":"Leaf item","completed":"Open","important":false,"folded":false,"items":[]}"#;
    let todo = todo_json(item);
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    focus_todo(&mut h);

    assert!(h.content_contains("Leaf item"));

    // Press Right on leaf — should open notes (focus moves away from todo)
    h.inject_key(KeyCode::Right, KeyMod::default());
    h.run_cycles(2);

    // Notes tab should appear (title "Notes" in screen)
    assert!(
        h.contains("Notes") || h.contains("note"),
        "Right on leaf should open notes"
    );
}
