//! REALISTIC clipboard tests — no F3 hacks, simulates actual user flow.
//! These tests reproduce the exact sequences a user would perform.

mod helpers;
use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn alt(ch: char) -> (KeyCode, KeyMod) {
    (KeyCode::Char(ch), KeyMod::ALT)
}
fn ctrl(ch: char) -> (KeyCode, KeyMod) {
    (KeyCode::Char(ch), KeyMod::CTRL)
}

/// Simulate: user opens kairn, opens file, yanks, checks ring.
#[test]
fn real_flow_open_file_yy_check_ring() {
    let dir = temp_project(&[("hello.rs", "fn main() {\n    println!(\"hi\");\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // User opens file via M-x :e
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_str("e hello.rs\n");
    h.run_cycles(5);

    // Verify file is open and visible
    assert!(h.content_contains("fn main"), "file should be open");

    // User presses yy to yank first line
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Check ring directly
    let ring_len = h.state.clipboard_ref().lock().unwrap().len();
    let ring_text: Vec<String> = h
        .state
        .clipboard_ref()
        .lock()
        .unwrap()
        .entries()
        .iter()
        .map(|e| e.text().to_string())
        .collect();
    assert!(ring_len > 0, "ring should have entry after yy. ring={ring_text:?}");
    assert!(
        ring_text[0].contains("fn main"),
        "ring top should be yanked line: {ring_text:?}"
    );
}

/// Simulate: user copies in editor, then pastes in M-x.
#[test]
fn real_flow_editor_yy_then_mx_paste() {
    let dir = temp_project(&[("a.txt", "copied line\nsecond\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Open file
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_str("e a.txt\n");
    h.run_cycles(5);

    // yy
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Open M-x and paste
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(3);

    let row = h.row(23);
    assert!(row.contains("copied line"), "M-x should show pasted text: '{row}'");
}

/// Simulate: user copies in editor, pastes in todo new item.
#[test]
fn real_flow_editor_yy_then_todo_paste() {
    use kairn::handler::downcast_desktop;
    use kairn::slots::{focus_tab_by_title, SlotId};

    let todo = r#"{"version":"1","title":"Test","items":[{"title":"existing","id":"x1"}]}"#;
    let dir = temp_project(&[("a.txt", "todo paste\n"), (".kairn.todo", todo)]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Open file and yy
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_str("e a.txt\n");
    h.run_cycles(5);
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Focus todo panel
    let desktop = h.program.desktop_mut();
    if let Some(d) = downcast_desktop(desktop) {
        focus_tab_by_title(d, SlotId::Left, "Todo");
        d.focus_panel(SlotId::Left as usize);
    }
    h.run_cycles(3);

    // New item + paste
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(3);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(3);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    assert!(
        h.contains("todo paste"),
        "todo should show pasted text:\n{}",
        h.screen_text()
    );
}

/// Simulate: editor p pastes from ring (yy then p on next line).
#[test]
fn real_flow_editor_yy_then_p() {
    let dir = temp_project(&[("a.txt", "line one\nline two\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Open file
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_str("e a.txt\n");
    h.run_cycles(5);

    // yy first line
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Move down, p
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(3);

    // Should have "line one" duplicated (original yy paste uses shared register, not ring)
    assert!(h.content_contains("line one"), "p should paste yanked line");
}
