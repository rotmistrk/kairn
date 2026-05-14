//! Scenario tests for todo tree UX improvements:
//! checkboxes, shift-arrow swap, new item opens editor with selection, indent-aware editor.

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

fn todo_json(items: &str) -> String {
    format!(r#"{{"version":"2.0","title":"Todo","items":[{items}]}}"#)
}

fn item(title: &str) -> String {
    format!(r#"{{"title":"{title}","completed":"Open","important":false,"folded":false,"items":[]}}"#)
}

fn done_item(title: &str) -> String {
    format!(r#"{{"title":"{title}","completed":"Done","important":false,"folded":false,"items":[]}}"#)
}

#[test]
fn todo_tree_renders_open_checkbox() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Buy milk"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    assert!(h.content_contains("[ ] Buy milk"));
}

#[test]
fn todo_tree_renders_done_checkbox() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&done_item("Done task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    assert!(h.content_contains("[x] Done task"));
}

#[test]
fn shift_down_swaps_items() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&format!("{},{}", item("First"), item("Second")));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Cursor on "First", Shift+Down should swap it with "Second"
    h.inject_key(
        KeyCode::Down,
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(2);

    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    let first_pos = content.find("First").unwrap();
    let second_pos = content.find("Second").unwrap();
    assert!(second_pos < first_pos, "Second should come before First after swap");
}

#[test]
fn shift_up_swaps_items() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&format!("{},{}", item("First"), item("Second")));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Move to "Second", then Shift+Up
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(
        KeyCode::Up,
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(2);

    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    let first_pos = content.find("First").unwrap();
    let second_pos = content.find("Second").unwrap();
    assert!(second_pos < first_pos, "Second should come before First after swap-up");
}

#[test]
fn new_sibling_opens_editor() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Existing"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);

    // The editor should be open with "<new task>" in the file
    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("<new task>"));

    // Type replaces the selected text
    h.inject_str("My item");
    h.run_cycles(2);
    assert!(h.content_contains("My item"));

    // Enter commits
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("My item"));
}

#[test]
fn new_child_opens_editor() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Parent"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('b'), KeyMod::default());
    h.run_cycles(2);

    // Check file was created with subtask
    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("<new task>"));

    // Esc cancels edit — title stays as "<new task>"
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);

    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("<new task>"));
}

#[test]
fn edit_existing_item() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Old title"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Press 'e' to edit
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);

    // Select all and replace
    h.inject_key(
        KeyCode::Home,
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(1);
    h.inject_str("New title");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    let content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(content.contains("New title"));
    assert!(!content.contains("Old title"));
}
