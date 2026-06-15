//! Tests for completion popup positioning with wrap and horizontal scroll.

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

fn ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), none());
    h.run_cycles(1);
    h.inject_str(cmd);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);
}

fn inject_completion(h: &mut TestHarness, labels: &[&str]) {
    let items: Vec<CompletionItem> = labels
        .iter()
        .map(|l| CompletionItem::new(l.to_string(), None, None, CompletionKind::Other))
        .collect();
    h.backend.inject(Event::Command {
        broadcast: false,
        id: CM_LSP_COMPLETION,
        data: Some(Box::new(items)),
    });
    h.run_cycles(1);
}

/// Popup appears near cursor in nowrap mode with h_scroll > 0.

#[test]
fn popup_position_with_hscroll() {
    // Line long enough to force h_scroll, with a space before the prefix word
    let long_line = format!("{} pre\n", "x =".repeat(20));
    let dir = temp_project(&[("f.rs", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");
    ex(&mut h, "set nowrap");
    h.run_cycles(2);

    // Move to end and enter insert mode (append)
    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(2);

    inject_completion(&mut h, &["prefix_match", "prefix_other"]);
    h.run_cycles(2);

    // Popup should be visible on screen (not off to the right)
    assert!(h.content_contains("prefix_match"));
}

/// Popup appears on wrapped line (cursor on second visual row).

#[test]
fn popup_position_on_wrapped_line() {
    // Line that wraps: content wider than editor, with word boundary before prefix
    let long_line = format!("{} pre\n", "x =".repeat(15));
    let dir = temp_project(&[("f.rs", &long_line)]);
    let mut h = TestHarness::with_size(dir.path(), 30, 10);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");
    ex(&mut h, "set wrap");

    // Move to end (which is on the wrapped second row) and enter insert
    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(2);

    inject_completion(&mut h, &["prefix_item", "prefix_two"]);
    h.run_cycles(2);

    // Popup should be visible
    assert!(h.content_contains("prefix_item"));
}

/// Popup near left edge with h_scroll (cursor moved back after scrolling right).

#[test]
fn popup_near_left_with_hscroll() {
    // Long line with a word near the end
    let content = format!("{} mid\n", "x =".repeat(25));
    let dir = temp_project(&[("f.rs", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 40, 10);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");
    ex(&mut h, "set nowrap");

    // Move to end to trigger h_scroll, then use 'A' to append
    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(2);

    inject_completion(&mut h, &["mid_word_complete"]);
    h.run_cycles(2);

    assert!(h.content_contains("mid_word_complete"));
}
