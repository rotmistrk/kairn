// === :e filename opens file in new tab ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn colon_e_opens_file_in_new_tab() {
    let dir = temp_project(&[
        ("hello.txt", "hello content"),
        ("target.txt", "UNIQUE_TARGET_CONTENT_XYZ"),
    ]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Verify we're looking at hello.txt
    assert!(h.contains("hello content"));
    // Type :e target.txt Enter
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e target.txt\n");
    h.run_cycles(1);
    // The new tab should show target.txt content
    assert!(
        h.contains("UNIQUE_TARGET_CONTENT_XYZ"),
        "expected target.txt content on screen after :e"
    );
}

#[test]
fn colon_e_tab_completes_filename() {
    let dir = temp_project(&[
        ("hello.txt", "hello"),
        ("Makefile", "all:"),
    ]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Type :e M then Tab to complete
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e M");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // The command prompt should show "e Makefile" (completed)
    // Look for the prompt line containing the completed text
    assert!(
        h.contains("e Makefile"),
        "expected Tab completion to show 'e Makefile' in command prompt"
    );
}
