//! Clipboard tests: Ctrl+C/V must work in all InputLines.

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

#[test]
fn ctrl_v_does_not_close_mx() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_key(KeyCode::Char('a'), KeyMod::default());
    h.run_cycles(3);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(3);
    assert!(h.row(23).contains(":"), "M-x stays active after Ctrl+V");
}

#[test]
fn ctrl_c_does_not_close_mx() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_key(KeyCode::Char('z'), KeyMod::default());
    h.run_cycles(3);
    h.inject_key(ctrl('a').0, ctrl('a').1);
    h.run_cycles(3);
    h.inject_key(ctrl('c').0, ctrl('c').1);
    h.run_cycles(3);
    assert!(h.row(23).contains(":"), "M-x stays active after Ctrl+C");
}

#[test]
fn clipboard_copy_paste_within_mx() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Open M-x, type "qw"
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_key(KeyCode::Char('q'), KeyMod::default());
    h.inject_key(KeyCode::Char('w'), KeyMod::default());
    h.run_cycles(3);

    // Select all, copy
    h.inject_key(ctrl('a').0, ctrl('a').1);
    h.run_cycles(3);
    h.inject_key(ctrl('c').0, ctrl('c').1);
    h.run_cycles(3);

    // Type 'X' (replaces selection)
    h.inject_key(
        KeyCode::Char('X'),
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(3);

    // Paste
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(3);

    // Should contain "Xqw" (X replaced selection, then paste appended "qw")
    let row = h.row(23);
    assert!(row.contains("qw"), "paste should insert copied text: '{row}'");
}
