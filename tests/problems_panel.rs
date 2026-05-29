//! Scenario tests for the Problems panel (LSP diagnostics view).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

/// Simulate a CM_DIAGNOSTIC command arriving.
fn push_diagnostic(h: &mut TestHarness, uri: &str, line: usize, msg: &str) {
    use kairn::commands::CM_DIAGNOSTIC;
    use kairn::lsp::diagnostics::{Diagnostic, Severity};
    let diags = vec![Diagnostic::new(line, 0, 5, Severity::Error, msg)];
    h.dispatch_command(CM_DIAGNOSTIC, Some(Box::new((uri.to_string(), diags))));
    h.run_cycles(2);
}

/// Focus the Problems tab via M-x problems.
fn focus_problems(h: &mut TestHarness) {
    h.inject_key(KeyCode::Char('≈'), KeyMod::default());
    h.inject_str("problems\n");
    h.run_cycles(2);
}

#[test]
fn problems_panel_shows_no_problems_initially() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    focus_problems(&mut h);
    assert!(h.content_contains("No problems"));
}

#[test]
fn problems_panel_shows_diagnostic() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    push_diagnostic(&mut h, "file:///a.rs", 0, "expected `;`");
    focus_problems(&mut h);
    assert!(h.content_contains("expected `;`"));
}

#[test]
fn problems_panel_clears_on_empty_diagnostics() {
    let dir = temp_project(&[("a.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    push_diagnostic(&mut h, "file:///a.rs", 0, "error here");
    focus_problems(&mut h);
    assert!(h.content_contains("error here"));
    // Clear diagnostics
    use kairn::commands::CM_DIAGNOSTIC;
    h.dispatch_command(
        CM_DIAGNOSTIC,
        Some(Box::new((
            "file:///a.rs".to_string(),
            Vec::<kairn::lsp::diagnostics::Diagnostic>::new(),
        ))),
    );
    h.run_cycles(2);
    assert!(h.content_contains("No problems"));
}
