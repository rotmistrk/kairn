//! Scenario tests for diagnostic lifecycle: dispatch, clear on edit, URI matching.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{CM_DIAGNOSTIC, CM_OPEN_FILE};
use kairn::lsp::diagnostics::{Diagnostic, Severity};
use txv_core::event::{Event, KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_file(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

fn inject_diagnostics(h: &mut TestHarness, uri: &str, diags: Vec<Diagnostic>) {
    h.backend.inject(Event::Command {
        broadcast: false,
        id: CM_DIAGNOSTIC,
        data: Some(Box::new((uri.to_string(), diags))),
    });
    h.run_cycles(1);
}

/// Diagnostics with matching URI are displayed (gutter marker visible).
#[test]
fn diagnostics_appear_for_matching_uri() {
    let dir = temp_project(&[("src/main.rs", "fn main() {\n    bad_call();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "src/main.rs");

    let uri = format!(
        "file://{}",
        dir.path().join("src/main.rs").canonicalize().unwrap().display()
    );
    inject_diagnostics(
        &mut h,
        &uri,
        vec![Diagnostic::new(1, 4, 12, Severity::Error, "not found")],
    );
    h.run_cycles(1);

    // The gutter marker should be visible
    assert!(h.content_contains("●"));
}

/// Diagnostics with non-matching URI are ignored.
#[test]
fn diagnostics_ignored_for_wrong_uri() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "src/main.rs");

    inject_diagnostics(
        &mut h,
        "file:///some/other/file.rs",
        vec![Diagnostic::new(0, 0, 5, Severity::Error, "wrong file")],
    );
    h.run_cycles(1);

    assert!(!h.content_contains("●"));
}

/// Editing clears diagnostics immediately.
#[test]
fn edit_clears_diagnostics() {
    let dir = temp_project(&[("f.rs", "fn f() { bad }\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_file(&mut h, dir.path(), "f.rs");

    let uri = format!("file://{}", dir.path().join("f.rs").canonicalize().unwrap().display());
    inject_diagnostics(&mut h, &uri, vec![Diagnostic::new(0, 9, 12, Severity::Error, "err")]);
    h.run_cycles(1);
    assert!(h.content_contains("●"));

    // Edit the file (type 'x' to delete char in normal mode)
    h.inject_key(KeyCode::Char('x'), none());
    h.run_cycles(2);

    // Diagnostics should be cleared
    assert!(!h.content_contains("●"));
}
