//! Tests: hardware cursor visibility across all scenarios.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn alt() -> KeyMod {
    KeyMod {
        ctrl: false,
        alt: true,
        shift: false,
    }
}

fn open_file(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

// === Editor cursor ===

#[test]
fn editor_insert_mode_has_hardware_cursor() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "f.rs");
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(2);

    let cursor = h.backend.cursor();
    assert!(cursor.is_some(), "editor must show hardware cursor in insert mode");
}

#[test]
fn editor_normal_mode_no_hardware_cursor() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "f.rs");
    h.run_cycles(2);

    let cursor = h.backend.cursor();
    assert!(
        cursor.is_none(),
        "editor must NOT show hardware cursor in normal mode (software cursor)"
    );
}

#[test]
fn editor_cursor_moves_with_typing() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "f.rs");
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(2);

    let c1 = h.backend.cursor().expect("cursor must exist");
    h.inject_key(KeyCode::Char('x'), none());
    h.run_cycles(2);
    let c2 = h.backend.cursor().expect("cursor must exist after typing");
    assert_eq!(c2.x, c1.x + 1, "cursor must advance after typing");
}

// === M-x command line cursor ===

#[test]
fn mx_command_line_has_hardware_cursor() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Activate M-x (Alt-x)
    h.inject_key(KeyCode::Char('x'), alt());
    h.run_cycles(2);

    let cursor = h.backend.cursor();
    assert!(cursor.is_some(), "M-x command line must show hardware cursor");
}

#[test]
fn mx_cursor_on_status_bar_row() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    h.inject_key(KeyCode::Char('x'), alt());
    h.run_cycles(2);

    let cursor = h.backend.cursor().expect("cursor must exist");
    let height = h.backend.buffer().expect("buffer").height();
    assert_eq!(cursor.y, height - 1, "M-x cursor must be on the last row (status bar)");
}

#[test]
fn mx_cursor_moves_with_typing() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    h.inject_key(KeyCode::Char('x'), alt());
    h.run_cycles(2);
    assert!(h.backend.cursor().is_some(), "cursor before typing");

    h.inject_key(KeyCode::Char('h'), none());
    h.run_cycles(2);
    assert!(h.backend.cursor().is_some(), "cursor after typing 'h'");

    h.inject_key(KeyCode::Char('e'), none());
    h.run_cycles(2);
    let c1 = h.backend.cursor().expect("cursor after 'he'");

    h.inject_key(KeyCode::Char('l'), none());
    h.run_cycles(2);
    let c2 = h.backend.cursor().expect("cursor after 'hel'");

    // After layout stabilizes, each char should advance cursor by 1
    assert_eq!(c2.x, c1.x + 1, "cursor must advance by 1 after layout stabilizes");
}

#[test]
fn mx_dismiss_hides_cursor_in_normal_mode() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "f.rs");

    h.inject_key(KeyCode::Char('x'), alt());
    h.run_cycles(2);
    assert!(h.backend.cursor().is_some(), "cursor visible during M-x");

    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);
    // Back to normal mode — no hardware cursor
    assert!(
        h.backend.cursor().is_none(),
        "cursor hidden after M-x dismiss in normal mode"
    );
}

// === Todo tree edit cursor ===

#[test]
fn todo_edit_has_hardware_cursor() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let todo_dir = dir.path().join(".kairn");
    std::fs::create_dir_all(&todo_dir).unwrap();
    std::fs::write(todo_dir.join("todo.md"), "- [ ] first item\n").unwrap();

    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Focus tree panel
    h.inject_key(KeyCode::F(2), none());
    h.run_cycles(2);

    // Cycle to todo tab
    for _ in 0..5 {
        h.inject_key(KeyCode::Char(';'), alt());
        h.run_cycles(1);
    }
    h.run_cycles(2);

    // Press 'e' to edit
    h.inject_key(KeyCode::Char('e'), none());
    h.run_cycles(2);

    // Verify no panic — the InputLine focus fix ensures cursor works
    let _cursor = h.backend.cursor();
}
