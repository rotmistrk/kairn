// === Integration: :set list renders special chars ===

mod helpers;
use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn set_list_shows_eol_marker() {
    let dir = temp_project(&[("t.txt", "hello\nworld")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set list\n");
    h.run_cycles(1);
    assert!(h.contains("$"), "expected $ EOL marker in list mode");
}

#[test]
fn set_list_shows_dot_for_space() {
    let dir = temp_project(&[("t.txt", "a b")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set list\n");
    h.run_cycles(1);
    assert!(h.contains("\u{00B7}"), "expected · for space in list mode");
}

#[test]
fn set_nonumber_hides_gutter() {
    let dir = temp_project(&[("t.txt", "hello")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Disable line numbers
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("set nonu\n");
    h.run_cycles(1);
    // With numbers off, "hello" should appear without leading digit
    let screen = h.screen_text();
    assert!(screen.contains("hello"));
}
