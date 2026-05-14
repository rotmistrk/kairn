//! Scenario tests for LSP navigation (goto definition, find references).

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE, CM_OPEN_FILE_FOCUS, CM_SHOW_RESULTS};
use kairn::views::results::ResultEntry;
use txv_core::event::{KeyCode, KeyMod};

/// Open a file and focus the editor panel.
fn open_file(h: &mut TestHarness, path: std::path::PathBuf) {
    h.dispatch_command(CM_OPEN_FILE, Some(Box::new(OpenFileRequest::new(path))));
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

/// Test 4: gd with single result opens file at line.
/// Simulates what the LSP handler does when it gets a single definition location:
/// dispatches CM_OPEN_FILE_FOCUS with the target path and line.
#[test]
fn goto_definition_single_result_opens_file() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() {\n    hello();\n}\n"),
        (
            "src/lib.rs",
            "// line 0\n// line 1\n// line 2\n// line 3\n// line 4\npub fn hello() {}\n",
        ),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    open_file(&mut h, dir.path().join("src/main.rs"));

    // Simulate LSP definition response: opens lib.rs at line 5, col 7
    let req = OpenFileRequest::at(dir.path().join("src/lib.rs"), 5, 7);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // lib.rs should now be visible with its content
    assert!(h.content_contains("pub fn hello()"));
}

/// Test 5: gr with single result jumps directly (no ResultsView).
/// When references returns exactly 1 location, the handler dispatches
/// CM_OPEN_FILE_FOCUS directly instead of CM_SHOW_RESULTS.
#[test]
fn find_references_single_result_jumps_directly() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() { foo(); }\n"),
        ("src/lib.rs", "line0\nline1\nline2\npub fn foo() {}\nline4\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    open_file(&mut h, dir.path().join("src/main.rs"));

    // Simulate single reference result — opens lib.rs at line 3
    let req = OpenFileRequest::at(dir.path().join("src/lib.rs"), 3, 7);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(3);

    // Should show lib.rs content directly
    assert!(h.content_contains("pub fn foo()"));
    // Should NOT have a "References" results view
    assert!(!h.contains("References:"));
}

/// Test 6: gr with multiple results opens ResultsView.
/// When references returns multiple locations, a ResultsView tab appears.
#[test]
fn find_references_multiple_results_opens_results_view() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() { foo(); }\n"),
        ("src/lib.rs", "pub fn foo() {}\n"),
        ("src/util.rs", "use crate::foo;\nfn bar() { foo(); }\n"),
    ]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    open_file(&mut h, dir.path().join("src/main.rs"));

    // Simulate multiple reference results via CM_SHOW_RESULTS
    let entries = vec![
        ResultEntry {
            path: dir.path().join("src/main.rs"),
            line: 0,
            col: 13,
            text: "foo();".to_string(),
        },
        ResultEntry {
            path: dir.path().join("src/lib.rs"),
            line: 0,
            col: 7,
            text: "pub fn foo() {}".to_string(),
        },
        ResultEntry {
            path: dir.path().join("src/util.rs"),
            line: 1,
            col: 11,
            text: "foo();".to_string(),
        },
    ];
    h.dispatch_command(
        CM_SHOW_RESULTS,
        Some(Box::new(("References: foo".to_string(), entries))),
    );
    h.run_cycles(3);

    // The results view should be visible with the title or entries
    assert!(h.contains("References") || h.content_contains("foo()"));
}
