//! Tests for visual rendering of completion dropdown (highlight changes).

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

fn row_bg_of(h: &TestHarness, text: &str) -> Option<txv_core::cell::Color> {
    let buf = h.backend.buffer()?;
    for y in 0..buf.height() {
        let row: String = (0..buf.width()).map(|x| buf.cell(x, y).ch()).collect();
        if let Some(col) = row.find(text) {
            return Some(buf.cell(col as u16, y).style().bg());
        }
    }
    None
}

#[test]
fn down_changes_visual_highlight() {
    let dir = temp_project(&[("f.rs", "fn f() {  }\n")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    h.inject_str("9l");
    h.run_cycles(1);
    enter_insert(&mut h);

    inject_completion(&mut h, make_items(&["alpha", "beta", "gamma"]));

    // Initially "alpha" is selected (highlighted)
    let alpha_bg_before = row_bg_of(&h, "alpha");
    let beta_bg_before = row_bg_of(&h, "beta");
    assert_ne!(
        alpha_bg_before, beta_bg_before,
        "selected item should have different bg than unselected"
    );

    // Press Down — "beta" becomes selected
    h.inject_key(KeyCode::Down, none());
    h.run_cycles(2);

    let alpha_bg_after = row_bg_of(&h, "alpha");
    let beta_bg_after = row_bg_of(&h, "beta");
    assert_ne!(
        alpha_bg_after, beta_bg_after,
        "after Down, selected item should still differ from unselected"
    );
    // The highlight moved: beta now has what alpha had before
    assert_eq!(
        beta_bg_after, alpha_bg_before,
        "beta should now have the selected highlight"
    );
    assert_eq!(
        alpha_bg_after, beta_bg_before,
        "alpha should now have the unselected bg"
    );
}
