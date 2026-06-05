//! Clipboard architecture tests: Ctrl+C/V must not close ModalKey.

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
    // Select: Home + Shift+End
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
    assert!(h.row(23).contains(":"), "M-x stays active after Ctrl+C");
}
