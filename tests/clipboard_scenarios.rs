//! Comprehensive clipboard scenario tests.
//! Every copy/paste combination + clipboard viewer verification.

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

fn open_file(h: &mut TestHarness, name: &str) {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in format!("e {name}").chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(5);
    // Focus center panel (editor) after M-x closes
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(3);
}

fn open_clipboard_viewer(h: &mut TestHarness) {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in "clipboard".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);
}

fn verify_ring_contains(h: &TestHarness, expected: &str) -> bool {
    if let Ok(ring) = h.state.clipboard_ref().lock() {
        ring.entries().iter().any(|e| e.text.contains(expected))
    } else {
        false
    }
}

// ─── Editor yy copies to ring ───────────────────────────────────────────

#[test]
fn editor_yy_appears_in_ring() {
    let dir = temp_project(&[("a.txt", "hello world\nsecond line\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    open_file(&mut h, "a.txt");

    // yy yanks the first line
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    assert!(
        verify_ring_contains(&h, "hello world"),
        "yy should push to clipboard ring"
    );
}

#[test]
fn editor_dd_appears_in_ring() {
    let dir = temp_project(&[("a.txt", "delete me\nkeep me\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    open_file(&mut h, "a.txt");

    // dd deletes+yanks the first line
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(3);

    assert!(
        verify_ring_contains(&h, "delete me"),
        "dd should push to clipboard ring"
    );
}

// ─── M-x Ctrl+C copies to ring ─────────────────────────────────────────

#[test]
fn mx_ctrl_c_appears_in_ring() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in "mxtext".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.run_cycles(3);
    // Select all: Home + Shift+End
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(
        KeyCode::End,
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(3);
    // Copy
    h.inject_key(ctrl('c').0, ctrl('c').1);
    h.run_cycles(3);

    assert!(verify_ring_contains(&h, "mxtext"), "Ctrl+C in M-x should push to ring");
}

// ─── Cross-widget paste from ring ───────────────────────────────────────

#[test]
fn editor_yy_paste_to_mx() {
    let dir = temp_project(&[("a.txt", "fromfile\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    open_file(&mut h, "a.txt");

    // yy
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Open M-x, paste
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    h.inject_key(ctrl('v').0, ctrl('v').1);
    h.run_cycles(3);

    let row = h.row(23);
    assert!(row.contains("fromfile"), "editor yy → M-x paste: '{row}'");
}

#[test]
fn mx_copy_paste_to_editor() {
    let dir = temp_project(&[("a.txt", "existing\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    open_file(&mut h, "a.txt");

    // Copy "injected" from M-x
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in "injected".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.run_cycles(3);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(
        KeyCode::End,
        KeyMod {
            shift: true,
            ..KeyMod::default()
        },
    );
    h.run_cycles(3);
    h.inject_key(ctrl('c').0, ctrl('c').1);
    h.run_cycles(3);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(3);

    // :paste into editor
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in "paste".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    assert!(h.content_contains("injected"), "M-x copy → :paste in editor");
}
