//! Full clipboard integration test suite.
//! Tests copy/paste between ALL input contexts: M-x, editor, todo title, todo notes.
//! These tests MUST run serially (they share the global internal clipboard).

mod helpers;
use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn ctrl(ch: char) -> (KeyCode, KeyMod) {
    (
        KeyCode::Char(ch),
        KeyMod {
            ctrl: true,
            ..KeyMod::default()
        },
    )
}
fn alt(ch: char) -> (KeyCode, KeyMod) {
    (
        KeyCode::Char(ch),
        KeyMod {
            alt: true,
            ..KeyMod::default()
        },
    )
}

/// Open M-x, type text, select all (Home + Shift+End), copy, close M-x.
fn copy_from_mx(h: &mut TestHarness, text: &str) {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    for ch in text.chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.run_cycles(2);
    // Select all: Home then Shift+End
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(
        KeyCode::End,
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(2);
    h.inject_key(ctrl('c').0, ctrl('c').1);
    h.run_cycles(2);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
}

/// Open M-x, paste, read content, close.
fn paste_to_mx(h: &mut TestHarness) -> String {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(2);
    let row = h.row(23);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    row
}

/// Focus editor (F3), paste via :paste command.
fn paste_to_editor(h: &mut TestHarness) {
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
    // Use :paste command (dispatches CM_CLIPBOARD_PASTE to editor)
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    for ch in "paste".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
}

/// Focus editor, yank current line (yy copies to shared register + system clipboard).
fn copy_from_editor(h: &mut TestHarness) {
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(2);
}

// ─── M-x → other targets ─────────────────────────────────────────────────

#[test]
fn mx_to_mx_roundtrip() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    copy_from_mx(&mut h, "mxtext");
    let row = paste_to_mx(&mut h);
    assert!(row.contains("mxtext"), "M-x → M-x paste failed: '{row}'");
}

#[test]
fn mx_to_editor() {
    let dir = temp_project(&[("a.txt", "existing\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open file in editor
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    for ch in "e a.txt".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // Copy from M-x
    copy_from_mx(&mut h, "frommx");

    // Paste to editor via :paste
    paste_to_editor(&mut h);

    assert!(
        h.content_contains("frommx"),
        "M-x → editor paste failed:\n{}",
        h.screen_text()
    );
}

// ─── Editor → other targets ──────────────────────────────────────────────

#[test]
fn editor_to_mx() {
    let dir = temp_project(&[("a.txt", "editorline\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open file
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    for ch in "e a.txt".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // yy to copy line to clipboard
    copy_from_editor(&mut h);

    // Paste into M-x
    let row = paste_to_mx(&mut h);
    assert!(row.contains("editorline"), "editor → M-x paste failed: '{row}'");
}

// ─── Todo tests ──────────────────────────────────────────────────────────

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
    format!(r#"{{"version":"1","title":"Test","items":[{items}]}}"#)
}

#[test]
fn mx_to_todo_edit() {
    let todo = todo_json(r#"{"title":"existing","id":"aaa"}"#);
    let dir = temp_project(&[("a.txt", ""), (".kairn.todo", &todo)]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Copy "fromMx" from M-x
    copy_from_mx(&mut h, "fromMx");
    h.run_cycles(2);

    // Focus todo, create new item (n), paste into title
    focus_todo(&mut h);
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(2);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    assert!(h.contains("fromMx"), "M-x → todo paste failed:\n{}", h.screen_text());
}

#[test]
fn editor_to_todo_edit() {
    let todo = todo_json(r#"{"title":"item1","id":"bbb"}"#);
    let dir = temp_project(&[("a.txt", "edline\n"), (".kairn.todo", &todo)]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open file and yank
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    for ch in "e a.txt".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    copy_from_editor(&mut h);

    // Focus todo, new item, paste
    focus_todo(&mut h);
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(2);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    assert!(h.contains("edline"), "editor → todo paste failed:\n{}", h.screen_text());
}
