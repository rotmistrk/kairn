//! Tests for UI toggle behavior on todo tree completion.
//!
//! - Space on parent with children → no-op
//! - Space on leaf → toggles and propagates to parent

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::mcp::commands::McpCommandQueue;
use txv_core::event::{KeyCode, KeyMod};
use txv_core::run::Waker;

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

const PARENT_CHILD_TODO: &str = r#"{
    "version": "2.0", "title": "Todo",
    "items": [{
        "title": "Parent", "completed": "Open", "important": false, "folded": false,
        "items": [
            {"title": "Child", "completed": "Open", "important": false, "folded": false, "items": []}
        ]
    }]
}"#;

#[test]
fn space_on_parent_is_noop() {
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), PARENT_CHILD_TODO);

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Space on parent (cursor is at parent, row 0)
    h.inject_key(KeyCode::Char(' '), KeyMod::default());
    h.run_cycles(2);

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Open);
}

#[test]
fn toggle_leaf_updates_parent_state() {
    let dir = temp_project(&[("dummy.txt", "")]);
    create_nested_todo(dir.path(), PARENT_CHILD_TODO);

    let mut h = TestHarness::new(dir.path());
    h.state.set_mcp_commands(McpCommandQueue::new(Waker::noop()));
    focus_todo(&mut h);

    // Navigate down to child and toggle it
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char(' '), KeyMod::default());
    h.run_cycles(2);

    let file = read_todo(dir.path());
    assert_eq!(file.items[0].items[0].completed, duir_core::model::Completion::Done);
    assert_eq!(file.items[0].completed, duir_core::model::Completion::Done);
}
