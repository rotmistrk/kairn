//! Clipboard viewer scenario tests.

mod helpers;
use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn ctrl(ch: char) -> (KeyCode, KeyMod) {
    (KeyCode::Char(ch), KeyMod::CTRL)
}
fn alt(ch: char) -> (KeyCode, KeyMod) {
    (KeyCode::Char(ch), KeyMod::ALT)
}

fn open_file(h: &mut TestHarness, name: &str) {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in format!("e {name}").chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(5);
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
// ─── Clipboard viewer shows entries ─────────────────────────────────────

#[test]
fn clipboard_viewer_shows_yy_entry() {
    let dir = temp_project(&[("a.txt", "viewer test line\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    open_file(&mut h, "a.txt");

    // yy
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Open clipboard viewer
    open_clipboard_viewer(&mut h);

    // Viewer should show the yanked text
    assert!(
        h.contains("viewer test line"),
        "clipboard viewer should show yanked text:\n{}",
        h.screen_text()
    );
}

#[test]
fn clipboard_viewer_shows_mx_copy() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Copy from M-x
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in "viewermx".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.run_cycles(3);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::End, KeyMod::SHIFT);
    h.run_cycles(3);
    h.inject_key(ctrl('c').0, ctrl('c').1);
    h.run_cycles(3);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(3);

    // Open clipboard viewer
    open_clipboard_viewer(&mut h);

    assert!(
        h.contains("viewermx"),
        "clipboard viewer should show M-x copied text:\n{}",
        h.screen_text()
    );
}

// ─── Named registers ────────────────────────────────────────────────────

#[test]
fn named_register_yank_and_paste() {
    let dir = temp_project(&[("a.txt", "reg line\nsecond\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);
    open_file(&mut h, "a.txt");

    // "ayy — yank to register 'a'
    h.inject_key(KeyCode::Char('"'), KeyMod::default());
    h.inject_key(KeyCode::Char('a'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Move to second line
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);

    // "ap — paste from register 'a'
    h.inject_key(KeyCode::Char('"'), KeyMod::default());
    h.inject_key(KeyCode::Char('a'), KeyMod::default());
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(3);

    assert!(
        h.content_contains("reg line"),
        "named register paste should insert text"
    );
}

// ─── Realistic: viewer open BEFORE copy ─────────────────────────────────

#[test]
fn clipboard_viewer_updates_after_copy() {
    let dir = temp_project(&[("a.txt", "live update\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(3);

    // Open clipboard viewer FIRST (empty)
    open_clipboard_viewer(&mut h);
    assert!(!h.contains("live update"), "viewer should be empty initially");

    // Now open file and yy
    open_file(&mut h, "a.txt");
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(3);

    // Switch back to clipboard viewer — should now show the entry
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(3);
    for ch in "clipboard".chars() {
        h.inject_key(KeyCode::Char(ch), KeyMod::default());
    }
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);

    assert!(
        h.contains("live update"),
        "viewer should show entry after yy:\n{}",
        h.screen_text()
    );
}
