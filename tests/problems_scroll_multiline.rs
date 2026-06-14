//! Regression test for bfe94dc: Problems panel scroll should account for
//! multi-line diagnostic messages when computing visibility.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[allow(dead_code)]
fn push_diagnostic(h: &mut TestHarness, uri: &str, line: usize, msg: &str) {
    use kairn::commands::CM_DIAGNOSTIC;
    use kairn::lsp::diagnostics::{Diagnostic, Severity};
    let diags = vec![Diagnostic::new(line, 0, 5, Severity::Error, msg)];
    h.dispatch_command(CM_DIAGNOSTIC, Some(Box::new((uri.to_string(), diags))));
    h.run_cycles(2);
}

#[allow(dead_code)]
fn push_diagnostics(h: &mut TestHarness, uri: &str, diags: Vec<(usize, &str)>) {
    use kairn::commands::CM_DIAGNOSTIC;
    use kairn::lsp::diagnostics::{Diagnostic, Severity};
    let ds: Vec<Diagnostic> = diags
        .into_iter()
        .map(|(line, msg)| Diagnostic::new(line, 0, 5, Severity::Error, msg))
        .collect();
    h.dispatch_command(CM_DIAGNOSTIC, Some(Box::new((uri.to_string(), ds))));
    h.run_cycles(2);
}

fn focus_problems(h: &mut TestHarness) {
    h.inject_key(KeyCode::Char('≈'), KeyMod::default());
    h.inject_str("problems\n");
    h.run_cycles(2);
}

/// Scrolling down past multi-line diagnostics should keep the cursor visible.
/// Before the fix, the old scroll logic assumed 1 line per entry and would
/// leave the cursor off-screen when expanded entries took multiple lines.
#[test]
fn scroll_with_multi_line_diagnostics() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    // Use a small viewport to force scrolling
    let mut h = TestHarness::with_size(dir.path(), 80, 10);
    h.run_cycles(2);

    // Push several diagnostics including multi-line messages
    let multi_line_msg = "error: mismatched types\n  expected `u32`\n  found `String`";
    push_diagnostics(
        &mut h,
        "file:///a.rs",
        vec![
            (0, multi_line_msg),
            (1, "unused variable"),
            (2, "another\nmulti-line\nerror"),
            (3, "simple error"),
            (4, "yet another error"),
        ],
    );

    focus_problems(&mut h);

    // Move down through all entries
    for _ in 0..4 {
        h.inject_key(KeyCode::Char('j'), KeyMod::default());
        h.run_cycles(1);
    }
    h.run_cycles(2);

    // The last entry should be visible after scrolling
    assert!(
        h.content_contains("yet another error"),
        "last diagnostic should be visible after scrolling past multi-line entries"
    );
}

/// Scrolling up from a later entry should make earlier multi-line entries visible.
#[test]
fn scroll_up_shows_multi_line_entry() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 10);
    h.run_cycles(2);

    let multi_line = "long\nmulti\nline\nerror\nmessage";
    push_diagnostics(
        &mut h,
        "file:///a.rs",
        vec![
            (0, multi_line),
            (1, "second"),
            (2, "third"),
            (3, "fourth"),
            (4, "fifth"),
        ],
    );

    focus_problems(&mut h);

    // Go to end
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.run_cycles(2);

    // Go back to top
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.run_cycles(2);

    // First entry text should be visible
    assert!(
        h.content_contains("long"),
        "first multi-line entry should be visible after scrolling to top"
    );
}
