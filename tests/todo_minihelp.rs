//! Scenario tests for todo tree minihelp (FocusGatedGroup) persistence.
//! These tests verify that the status bar minihelp appears when the todo panel
//! is focused and remains visible after various operations.

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

fn status_bar(h: &TestHarness) -> String {
    h.row(23)
}

fn has_minihelp(h: &TestHarness) -> bool {
    let bar = status_bar(h);
    bar.contains("prio") || bar.contains("+prio")
}

#[test]
fn minihelp_appears_on_todo_focus() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);
    h.run_cycles(2);

    assert!(
        has_minihelp(&h),
        "minihelp should appear when todo focused: [{}]",
        status_bar(&h)
    );
}

#[test]
fn minihelp_persists_after_edit_commit() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);
    assert!(has_minihelp(&h), "precondition: minihelp visible");

    // Start edit
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);
    // Minihelp should be hidden during edit
    assert!(!has_minihelp(&h), "minihelp should hide during edit");

    // Commit edit with Enter
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    assert!(
        has_minihelp(&h),
        "minihelp should reappear after edit commit: [{}]",
        status_bar(&h)
    );
}

#[test]
fn minihelp_persists_after_edit_cancel() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);
    assert!(has_minihelp(&h), "precondition: minihelp visible");

    // Start edit
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);

    // Cancel edit with Esc
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(3);

    assert!(
        has_minihelp(&h),
        "minihelp should reappear after edit cancel: [{}]",
        status_bar(&h)
    );
}

#[test]
fn minihelp_persists_after_new_sibling() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);
    assert!(has_minihelp(&h), "precondition: minihelp visible");

    // New sibling (opens editor)
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);

    // Commit the new item
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    assert!(
        has_minihelp(&h),
        "minihelp should reappear after new sibling commit: [{}]",
        status_bar(&h)
    );
}

#[test]
fn minihelp_persists_after_new_child() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);
    assert!(has_minihelp(&h), "precondition: minihelp visible");

    // New child (opens editor)
    h.inject_key(KeyCode::Char('b'), KeyMod::default());
    h.run_cycles(2);

    // Commit
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    assert!(
        has_minihelp(&h),
        "minihelp should reappear after new child commit: [{}]",
        status_bar(&h)
    );
}

#[test]
fn hotkeys_work_after_edit_commit() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);

    // Edit and commit
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    // Toggle in-progress with 'i' — should work (hotkey active)
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(2);

    // Verify the status changed (▸ = in progress)
    assert!(
        h.content_contains("▸"),
        "hotkey 'i' should toggle in-progress after edit: screen has no ▸"
    );
}

#[test]
fn minihelp_disappears_on_focus_away() {
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item("Task"));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    focus_todo(&mut h);
    assert!(has_minihelp(&h), "precondition: minihelp visible");

    // Focus editor (F3)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    assert!(
        !has_minihelp(&h),
        "minihelp should disappear when todo loses focus: [{}]",
        status_bar(&h)
    );
}
