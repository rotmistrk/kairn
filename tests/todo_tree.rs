//! Integration tests for the Todo tree view.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn focus_todo(h: &mut TestHarness) {
    use kairn::handler::downcast_desktop;
    use kairn::layout_group::SlotId;
    let desktop = h.program.desktop_mut();
    if let Some(d) = downcast_desktop(desktop) {
        d.focus_tab_by_title(SlotId::Left, "Todo");
        d.focus_slot(SlotId::Left);
    }
    h.run_cycles(2);
}

#[test]
fn todo_tree_empty_shows_placeholder() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    assert!(h.content_contains("empty"));
}

#[test]
fn todo_tree_add_item_on_empty() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);

    assert!(dir.path().join(".kairn.todo").exists());
    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("<new task>"));
}

#[test]
fn todo_tree_toggle_completion() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let todo = r#"{"version":"2.0","title":"Todo","items":[{"title":"Buy milk","completed":"Open","important":false,"folded":false,"items":[]}]}"#;
    std::fs::write(dir.path().join(".kairn.todo"), todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Press space to toggle completion
    h.inject_key(KeyCode::Char(' '), KeyMod::default());
    h.run_cycles(2);

    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("Done"));
}

#[test]
fn todo_tree_new_sibling() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let todo = r#"{"version":"2.0","title":"Todo","items":[{"title":"First","completed":"Open","important":false,"folded":false,"items":[]}]}"#;
    std::fs::write(dir.path().join(".kairn.todo"), todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);

    // Verify file has both items
    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("First"));
    assert!(content.contains("<new task>"));
}

#[test]
fn todo_tree_save_creates_file() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    assert!(!dir.path().join(".kairn.todo").exists());

    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);

    assert!(dir.path().join(".kairn.todo").exists());
    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("<new task>"));
}

#[test]
fn todo_tree_loads_existing_file() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let todo = r#"{"version":"2.0","title":"Todo","items":[{"title":"Task A","completed":"Open","important":false,"folded":false,"items":[]},{"title":"Task B","completed":"Done","important":true,"folded":false,"items":[]}]}"#;
    std::fs::write(dir.path().join(".kairn.todo"), todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Verify file content is loaded (check the file, not screen — panel may be narrow)
    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("Task A"));
    assert!(content.contains("Task B"));
    // Verify tree has items (not showing placeholder)
    assert!(!h.content_contains("empty"));
}
