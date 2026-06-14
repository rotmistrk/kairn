//! Test: LSP references results show relative paths and context text.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::CM_SHOW_RESULTS;
use kairn::views::results::ResultEntry;
#[allow(unused_imports)]
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn references_show_relative_paths_and_context() {
    let dir = temp_project(&[
        ("src/main.rs", "fn main() { greet(); }\n"),
        ("src/lib.rs", "pub fn greet() { println!(\"hi\"); }\n"),
    ]);
    let mut h = TestHarness::with_size(dir.path(), 100, 24);
    h.run_cycles(1);

    // Simulate LSP references result with absolute paths and context text
    let entries = vec![
        ResultEntry::new(dir.path().join("src/main.rs"), 0, 13, "greet();".to_string()),
        ResultEntry::new(dir.path().join("src/lib.rs"), 0, 7, "pub fn greet()".to_string()),
    ];
    h.dispatch_command(
        CM_SHOW_RESULTS,
        Some(Box::new(("References: greet".to_string(), entries))),
    );
    h.run_cycles(2);

    // Should show relative paths (not absolute)
    assert!(
        h.content_contains("src/main.rs"),
        "should show relative path src/main.rs"
    );
    assert!(
        !h.content_contains(&dir.path().to_string_lossy().to_string()),
        "should NOT show absolute path prefix"
    );
    // Should show context text
    assert!(h.content_contains("greet()"), "should show context text from entries");
}
