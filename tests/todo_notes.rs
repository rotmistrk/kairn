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
fn open_note_sets_state_and_tab() {
    let dir = temp_project(&[("dummy.txt", "")]);
    std::fs::write(dir.path().join(".kairn.todo"), todo_json("Task", "hello world")).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Press N to open note
    h.inject_key(KeyCode::Char('N'), KeyMod::default());
    h.run_cycles(4);

    // Verify state has the tree path stored
    assert_eq!(h.state.todo_note_path, Some(vec![0]));

    // Verify Notes tab is active with the note content in the buffer
    use kairn::handler::downcast_desktop;
    use kairn::layout_group::SlotId;
    let desktop = downcast_desktop(h.program.desktop_mut()).unwrap();
    assert_eq!(desktop.active_tab_title(SlotId::Center), Some("Notes"));
    let nv = desktop
        .find_view_mut::<kairn::views::notes::NotesView>(SlotId::Center)
        .unwrap();
    assert_eq!(nv.content(), "hello world");
}

#[test]
fn open_note_on_empty_note() {
    let dir = temp_project(&[("dummy.txt", "")]);
    std::fs::write(dir.path().join(".kairn.todo"), todo_json("Task", "")).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    h.inject_key(KeyCode::Char('N'), KeyMod::default());
    h.run_cycles(4);

    use kairn::handler::downcast_desktop;
    use kairn::layout_group::SlotId;
    let desktop = downcast_desktop(h.program.desktop_mut()).unwrap();
    let nv = desktop
        .find_view_mut::<kairn::views::notes::NotesView>(SlotId::Center)
        .unwrap();
    assert_eq!(nv.content(), "");
}

#[test]
fn save_note_writes_back_to_todo_json() {
    let dir = temp_project(&[("dummy.txt", "")]);
    std::fs::write(dir.path().join(".kairn.todo"), todo_json("Task", "original")).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Open note
    h.inject_key(KeyCode::Char('N'), KeyMod::default());
    h.run_cycles(4);

    // Type new content (replaces buffer since it starts empty-ish)
    // First select all, then type
    h.inject_key(
        KeyCode::Home,
        KeyMod {
            ctrl: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(1);
    // Enter insert mode and replace content
    h.inject_key(KeyCode::Char('c'), KeyMod::default()); // c in normal mode needs motion
                                                         // Simpler: use ex command to replace
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("%d");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Enter insert mode and type
    h.inject_key(KeyCode::Char('i'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("updated note");
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);

    // Save via :w
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("w");
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(4);

    // Verify the todo file was updated
    let todo_content = std::fs::read_to_string(dir.path().join(".kairn.todo")).unwrap();
    assert!(
        todo_content.contains("updated note"),
        "todo should contain 'updated note': {todo_content}"
    );
}
