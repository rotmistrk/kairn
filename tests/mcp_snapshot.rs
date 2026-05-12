//! MCP snapshot collection tests.

mod helpers;

use helpers::{TestHarness, temp_project};
use std::sync::{Arc, Mutex};

use kairn::mcp::collect::collect_snapshot;
use kairn::mcp::snapshot::McpSnapshot;

#[test]
fn mcp_snapshot_collects_editor_tabs() {
    let dir = temp_project(&[("src/main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());

    // Open a file
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(
            dir.path().join("src/main.rs"),
        ))),
    );
    h.run_cycles(2);

    // Collect snapshot directly from the desktop
    let desktop = h.program.desktop_mut();
    let lg = desktop
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<kairn::layout_group::LayoutGroup>())
        .expect("desktop is LayoutGroup");
    let snap = collect_snapshot(lg);

    assert!(!snap.tabs.is_empty(), "Expected tabs in snapshot");
    let has_editor = snap.tabs.iter().any(|t| t.tab_type == "editor");
    assert!(has_editor, "Expected an editor tab in snapshot");
}

#[test]
fn mcp_snapshot_handler_updates_arc() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    let snap = Arc::new(Mutex::new(McpSnapshot::default()));
    h.state.mcp_snapshot = Some(Arc::clone(&snap));

    // Open file to generate commands
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.path().join("a.txt")))),
    );

    // Run many cycles — each dispatch_command call increments mcp_tick
    for _ in 0..25 {
        h.dispatch_command(kairn::commands::CM_CURSOR_MOVED, Some(Box::new((0u32, 0u32))));
    }

    let locked = snap.lock().unwrap();
    assert!(!locked.tabs.is_empty(), "Expected snapshot populated after 20+ commands");
}

#[test]
fn mcp_snapshot_no_panic_without_arc() {
    let dir = temp_project(&[("a.txt", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    // No mcp_snapshot set — should not panic
    h.run_cycles(25);
}

