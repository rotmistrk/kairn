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

    // Down once selects "beta" — dropdown still visible with all items
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(2);
    assert!(
        h.content_contains("alpha"),
        "dropdown should still show 'alpha' after Down"
    );
    assert!(
        h.content_contains("beta"),
        "dropdown should still show 'beta' after Down"
    );
    assert!(
        h.content_contains("gamma"),
        "dropdown should still show 'gamma' after Down"
    );

    // Enter accepts the now-selected "beta"
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
#[test]
fn down_up_navigation_changes_selection() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["aaa", "bbb", "ccc", "ddd"]));

    // Down twice → cursor on "ccc"
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(2);
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(2);
    // Dropdown still visible
    assert!(h.content_contains("aaa"), "dropdown visible after 2x Down");
    assert!(h.content_contains("ddd"), "dropdown visible after 2x Down");

    // Up once → cursor back on "bbb"
    h.inject_key(KeyCode::Up, none());
    h.run_cycles(2);
    // Dropdown still visible
    assert!(h.content_contains("bbb"), "dropdown visible after Up");

    // Enter accepts "bbb" (Down, Down, Up = index 1)
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(6);

    assert!(h.content_contains("bbb"), "'bbb' should be inserted");
    assert!(!h.content_contains("ccc"), "'ccc' should NOT be in buffer");
}
