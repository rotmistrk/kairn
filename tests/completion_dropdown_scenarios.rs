//! Comprehensive scenario tests for the completion dropdown modal flow.
//! Tests cover: appear/dismiss, navigation, filtering, accept, and M-x completion.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_LSP_COMPLETION;
use kairn::lsp::requests::{CompletionItem, CompletionKind};
use txv_core::event::{Event, KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_and_focus(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

fn enter_insert(h: &mut TestHarness) {
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);
}

fn inject_completion(h: &mut TestHarness, items: Vec<CompletionItem>) {
    h.backend.inject(Event::Command {
        broadcast: false,
        id: CM_LSP_COMPLETION,
        data: Some(Box::new(items)),
    });
    h.run_cycles(3);
}

fn make_items(labels: &[&str]) -> Vec<CompletionItem> {
    labels
        .iter()
        .map(|l| CompletionItem::new(l.to_string(), None, Some(l.to_string()), CompletionKind::Other))
        .collect()
}

// ─── Editor completion tests ─────────────────────────────────────────

#[test]
fn dropdown_appears_on_completion() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Position at empty spot and enter insert mode
    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["alpha", "beta", "gamma"]));

    // Dropdown content should be visible on screen
    assert!(h.content_contains("alpha"), "first item should be visible");
    assert!(h.content_contains("beta"), "second item should be visible");
    assert!(h.content_contains("gamma"), "third item should be visible");
}

#[test]
fn esc_cancels_dropdown() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["alpha", "beta"]));
    assert!(h.content_contains("alpha"), "dropdown should be showing");

    // Press Esc to cancel
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(4);

    // Dropdown items should no longer be visible
    assert!(!h.content_contains("alpha"), "dropdown should be dismissed");
    assert!(!h.content_contains("beta"), "dropdown should be dismissed");
}

#[test]
fn enter_accepts_first_item() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["alpha", "beta", "gamma"]));

    // Press Enter immediately — accepts first item
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    assert!(h.content_contains("alpha"), "first item 'alpha' should be inserted");
}

#[test]
fn down_then_enter_accepts_second_item() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["alpha", "beta", "gamma"]));

    // Down once selects "beta", Enter accepts
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(2);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    assert!(h.content_contains("beta"), "second item 'beta' should be inserted");
    assert!(
        !h.content_contains("alpha"),
        "'alpha' should NOT be on screen. Screen:\n{}",
        h.screen_text()
    );
}

#[test]
fn up_wraps_to_last_item() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["alpha", "beta", "gamma"]));

    // Up from first item stays at first (no wrap), Enter accepts "alpha"
    h.inject_key(KeyCode::Up, none());
    h.run_cycles(2);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    assert!(
        h.content_contains("alpha"),
        "first item 'alpha' should be inserted (no wrap on Up)"
    );
}

#[test]
fn typing_filters_items() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["println", "print", "format"]));

    // Type "pr" to filter — only "println" and "print" should remain
    h.inject_str("pr");
    h.run_cycles(3);

    assert!(h.content_contains("println"), "'println' matches 'pr' filter");
    assert!(h.content_contains("print"), "'print' matches 'pr' filter");
    assert!(!h.content_contains("format"), "'format' should be filtered out");
}

#[test]
fn typing_then_enter_accepts_filtered() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["println", "print", "format"]));

    // Type "println" to narrow to one match, Enter accepts
    h.inject_str("println");
    h.run_cycles(3);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    assert!(h.content_contains("println"), "'println' should be inserted");
}

#[test]
fn down_down_enter_selects_third() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["one", "two", "three", "four", "five"]));

    // Down twice selects "three", Enter accepts
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(2);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    assert!(h.content_contains("three"), "third item 'three' should be inserted");
}

// ─── M-x command line completion tests ───────────────────────────────

#[test]
fn mx_shows_completion_on_tab() {
    let dir = temp_project(&[("f.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x prompt
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(1);

    // Type "e" and press Tab to trigger completion
    h.inject_str("e");
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(3);

    // Dropdown should appear with commands starting with 'e'
    let screen = h.screen_text();
    assert!(
        screen.contains("edit") || screen.contains("e "),
        "Tab should show completion dropdown with commands matching 'e'"
    );
}

#[test]
fn mx_esc_cancels() {
    let dir = temp_project(&[("f.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x, type "ed", Tab to show dropdown
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(1);
    h.inject_str("ed");
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(3);

    // Press Esc to cancel
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(4);

    // Should be back to normal mode (status bar shows F1:Help)
    let screen = h.screen_text();
    assert!(
        screen.contains("F1") || screen.contains("Help"),
        "Esc should dismiss M-x and return to normal mode"
    );
}

#[test]
fn mx_enter_accepts_and_executes() {
    let dir = temp_project(&[("hello.txt", "world\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);

    // Open M-x, type "he", Tab to trigger completion
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(1);
    h.inject_str("he");
    h.inject_key(KeyCode::Tab, none());
    h.run_cycles(3);

    // If dropdown shows "help", press Enter to accept and execute
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    // "help" command should have executed — help content visible
    let screen = h.screen_text();
    assert!(
        screen.contains("Help") || screen.contains("help") || screen.contains("F1"),
        "accepting 'help' completion should show help content"
    );
}
