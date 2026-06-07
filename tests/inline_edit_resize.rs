//! Regression test for 9f9698c: Inline edit in todo/struct/csv views should be
//! committed (not lost) when the view is resized.

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

fn item(title: &str) -> String {
    format!(r#"{{"title":"{title}","completed":"Open","important":false,"folded":false,"items":[]}}"#)
}

/// When editing a todo item title and a resize event occurs, the edit should be committed.
#[test]
fn todo_inline_edit_committed_on_resize() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Original"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    focus_todo(&mut h);

    assert!(h.content_contains("Original"));

    // Press 'e' to start editing
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);

    // Select all and type new text
    h.inject_key(KeyCode::Char('a'), KeyMod::CTRL);
    h.run_cycles(1);
    h.inject_str("Updated");
    h.run_cycles(2);

    // Simulate resize (inject a Resize event)
    h.backend.inject(txv_core::event::Event::Resize(100, 20));
    h.run_cycles(3);

    // The edit should have been committed — "Updated" persists
    assert!(h.content_contains("Updated"), "inline edit must be committed on resize");
    assert!(!h.content_contains("Original"), "old title should be replaced");
}
/// Struct view: resize during inline edit should cancel (discard) the edit
/// without crashing. The original value is preserved.
#[test]
fn struct_view_inline_edit_cancelled_on_resize() {
    let dir = temp_project(&[("test.json", r#"{"name":"old_value"}"#)]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open JSON from tree (same pattern as struct_view_edit.rs)
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    // Navigate to value: j to "name" scalar, Tab to Value column, Enter to edit
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // Select all and type new value
    h.inject_key(KeyCode::Char('a'), KeyMod::CTRL);
    h.run_cycles(1);
    h.inject_str("new_value");
    h.run_cycles(2);

    // Resize — should cancel the edit (struct view discards on resize)
    h.backend.inject(txv_core::event::Event::Resize(100, 20));
    h.run_cycles(3);

    // The original value should be preserved (edit discarded)
    assert!(
        h.content_contains("old_value"),
        "struct view should preserve original value after resize cancels edit"
    );
}
