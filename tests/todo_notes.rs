//! Integration tests for the Todo notes pane (open_todo_note / sync_todo_note).

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

fn todo_json(title: &str, note: &str) -> String {
    format!(
        r#"{{"version":"2.0","title":"Todo","items":[{{"title":"{}","completed":"Open","important":false,"folded":false,"note":"{}","items":[]}}]}}"#,
        title, note
    )
}

#[test]
fn open_note_creates_file_and_tab() {
    let dir = temp_project(&[("dummy.txt", "")]);
    std::fs::write(dir.path().join(".kairn.todo"), todo_json("Task", "hello world")).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Press N to open note
    h.inject_key(KeyCode::Char('N'), KeyMod::default());
    h.run_cycles(4);

    // Verify .kairn/note.md was created with the note content
    let note_file = dir.path().join(".kairn").join("note.md");
    assert!(note_file.exists(), ".kairn/note.md should exist");
    let content = std::fs::read_to_string(&note_file).unwrap();
    assert_eq!(content, "hello world");

    // Verify state has the tree path stored
    assert_eq!(h.state.todo_note_path, Some(vec![0]));
}

#[test]
fn open_note_on_empty_note_creates_empty_file() {
    let dir = temp_project(&[("dummy.txt", "")]);
    std::fs::write(dir.path().join(".kairn.todo"), todo_json("Task", "")).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('N'), KeyMod::default());
    h.run_cycles(4);

    let note_file = dir.path().join(".kairn").join("note.md");
    assert!(note_file.exists());
    let content = std::fs::read_to_string(&note_file).unwrap();
    assert_eq!(content, "");
}

#[test]
fn sync_todo_note_writes_back_to_item() {
    let dir = temp_project(&[("dummy.txt", "")]);
    std::fs::write(dir.path().join(".kairn.todo"), todo_json("Task", "original")).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Open note
    h.inject_key(KeyCode::Char('N'), KeyMod::default());
    h.run_cycles(4);

    // Simulate editing: overwrite the note file with new content
    let note_file = dir.path().join(".kairn").join("note.md");
    std::fs::write(&note_file, "updated note").unwrap();

    // Trigger sync (happens on CM_SAVE)
    kairn::handler_drain::sync_todo_note(&mut h.state);

    // Verify the todo file was updated
    let todo_content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(todo_content.contains("updated note"));
    assert!(!todo_content.contains("original"));
}
