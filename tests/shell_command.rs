// === Feature 3: :!command execution ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn bang_echo_shows_output_in_new_tab() {
    let dir = temp_project(&[("t.txt", "original")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Run :!echo UNIQUE_OUTPUT
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("!echo UNIQUE_OUTPUT\n");
    h.run_cycles(1);
    // Output should appear on screen in a new tab
    assert!(
        h.contains("UNIQUE_OUTPUT"),
        "expected shell output 'UNIQUE_OUTPUT' visible on screen"
    );
}

#[test]
fn range_bang_sort_sorts_lines() {
    let dir = temp_project(&[("t.txt", "cherry\napple\nbanana\ndate")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Sort lines 1-3: :1,3!sort
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("1,3!sort\n");
    h.run_cycles(1);
    // After sorting lines 1-3 (cherry, apple, banana), expect: apple, banana, cherry
    let screen = h.screen_text();
    let apple_pos = screen.find("apple").expect("apple should be on screen");
    let banana_pos = screen.find("banana").expect("banana should be on screen");
    let cherry_pos = screen.find("cherry").expect("cherry should be on screen");
    assert!(apple_pos < banana_pos, "apple should come before banana");
    assert!(banana_pos < cherry_pos, "banana should come before cherry");
    // date should remain unchanged
    assert!(h.contains("date"));
}
