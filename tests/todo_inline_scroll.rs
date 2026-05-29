//! Scenario test: inline editor scrolls to keep cursor visible in narrow panel.

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

#[test]
fn edit_long_title_shows_cursor_at_end() {
    let long_title = "This is a very long todo item title that exceeds the panel width";
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item(long_title));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Press 'e' to edit — cursor should be at end with text selected
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(3);

    // The end of the title should be visible (scrolled), not the beginning
    let screen = h.screen_text();
    assert!(
        screen.contains("l width"),
        "End of long title should be visible when cursor is at end. Got:\n{screen}"
    );
    // The beginning should NOT be visible
    assert!(
        !screen.contains("This is a"),
        "Start of title should NOT be visible when scrolled to end"
    );
}

#[test]
fn edit_long_title_left_arrow_scrolls_back() {
    let long_title = "ABCDEFGHIJ_start_middle_end_of_long_title_here";
    let dir = temp_project(&[("x.txt", "")]);
    let todo = todo_json(&item(long_title));
    std::fs::write(dir.path().join(".kairn.todo"), &todo).unwrap();

    let mut h = TestHarness::new(dir.path());
    focus_todo(&mut h);

    // Edit
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(2);

    // Press Home to go to start
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(2);

    // Beginning should now be visible
    let content = h.screen_text();
    assert!(
        content.contains("ABCDEFGH"),
        "Start of title should be visible after Home. Got:\n{content}"
    );
}
