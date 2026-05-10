// === Tab completion edge cases ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn tab_no_match_does_nothing() {
    let dir = temp_project(&[("hello.txt", "hi")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e ZZZ");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // No match — buffer should stay as "e ZZZ"
    assert!(h.contains("e ZZZ"), "no match should leave buffer unchanged");
}

#[test]
fn tab_multiple_matches_does_not_complete() {
    let dir = temp_project(&[
        ("main.rs", "fn main(){}"),
        ("mod.rs", "mod x;"),
    ]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e m");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Multiple matches (main.rs, mod.rs) — should not complete
    // Buffer stays as "e m"
    let screen = h.screen_text();
    // Should NOT have completed to either full name
    assert!(
        !screen.contains("e main.rs") && !screen.contains("e mod.rs"),
        "multiple matches should not auto-complete"
    );
}

#[test]
fn tab_on_non_edit_command_does_nothing() {
    let dir = temp_project(&[("t.txt", "hi")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set ");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Tab on :set should not crash or change buffer
    // Note: trailing space may be trimmed by screen_text()
    assert!(h.contains(":set"));
}
