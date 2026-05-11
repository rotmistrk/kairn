// === gR enters command mode with lsp-rename pre-filled ===

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_file_and_focus(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.inject_key(KeyCode::F(3), KeyMod::default());
}

#[test]
fn g_r_prefills_lsp_rename_with_word() {
    let dir = temp_project(&[("test.rs", "fn hello_world() {}")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Move to 'h' of hello_world (col 3)
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(1);
    // Press gR
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.inject_key(KeyCode::Char('R'), KeyMod::default());
    h.run_cycles(1);
    // Should show command prompt with "lsp-rename hello_world"
    assert!(
        h.contains("lsp-rename hello_world"),
        "expected 'lsp-rename hello_world' in prompt, got: {}",
        h.screen_text()
    );
}

#[test]
fn g_r_on_non_word_shows_empty_name() {
    let dir = temp_project(&[("test.rs", "  spaces")]);
    let mut h = TestHarness::new(dir.path());
    open_file_and_focus(&mut h);
    h.run_cycles(1);
    // Cursor at col 0 which is a space
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.inject_key(KeyCode::Char('R'), KeyMod::default());
    h.run_cycles(1);
    // Should show "lsp-rename " (empty word)
    assert!(
        h.contains("lsp-rename"),
        "expected 'lsp-rename' in prompt, got: {}",
        h.screen_text()
    );
}
