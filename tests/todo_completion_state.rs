//! Tests for todo tree parent completion state computed from descendants on file load.
//!
//! When loading a todo file, parent completion states should be reconciled
//! to match the actual state of their descendants.

mod helpers;

use helpers::{temp_project, TestHarness};

fn create_nested_todo(dir: &std::path::Path, structure: &str) {
    std::fs::write(dir.join(".kairn.todo"), structure).unwrap();
}

fn read_todo(dir: &std::path::Path) -> duir_core::TodoFile {
    let content = std::fs::read_to_string(dir.join(".kairn.todo")).unwrap();
    serde_json::from_str(&content).unwrap()
}

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

#[test]
fn all_children_done_parent_becomes_done() {
    let todo = r#"{
        "version": "2.0", "title": "Todo",
        "items": [{
            "title": "Parent", "completed": "Open", "important": false, "folded": false,
            "items": [
                {"title": "Child 1", "completed": "Done", "important": false, "folded": false, "items": []},
                {"title": "Child 2", "completed": "Done", "important": false, "folded": false, "items": []}
            ]
        }]
    }"#;
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), todo);

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);
    h.run_cycles(2);

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Done);
}

#[test]
fn no_children_done_parent_is_open() {
    let todo = r#"{
        "version": "2.0", "title": "Todo",
        "items": [{
            "title": "Parent", "completed": "Done", "important": false, "folded": false,
            "items": [
                {"title": "Child 1", "completed": "Open", "important": false, "folded": false, "items": []},
                {"title": "Child 2", "completed": "Open", "important": false, "folded": false, "items": []}
            ]
        }]
    }"#;
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), todo);

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);
    h.run_cycles(2);

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Open);
}

#[test]
fn mixed_children_parent_is_partial() {
    let todo = r#"{
        "version": "2.0", "title": "Todo",
        "items": [{
            "title": "Parent", "completed": "Open", "important": false, "folded": false,
            "items": [
                {"title": "Child 1", "completed": "Done", "important": false, "folded": false, "items": []},
                {"title": "Child 2", "completed": "Open", "important": false, "folded": false, "items": []}
            ]
        }]
    }"#;
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), todo);

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);
    h.run_cycles(2);

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Partial);
}

#[test]
fn nested_hierarchy_propagates_up() {
    let todo = r#"{
        "version": "2.0", "title": "Todo",
        "items": [{
            "title": "Grandparent", "completed": "Open", "important": false, "folded": false,
            "items": [{
                "title": "Parent", "completed": "Open", "important": false, "folded": false,
                "items": [
                    {"title": "Child", "completed": "Done", "important": false, "folded": false, "items": []}
                ]
            }]
        }]
    }"#;
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), todo);

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);
    h.run_cycles(2);

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Done);
    assert_eq!(file.items[0].items[0].completed, duir_core::model::Completion::Done);
}
