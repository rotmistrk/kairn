//! Test: editor goto pre-sets viewport_scroll when bounds not yet set.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
#[allow(unused_imports)]
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn goto_prescrolls_when_bounds_not_set() {
    // File with 100 lines — opening at line 80 should show that line on first draw
    let content: String = (1..=100).map(|i| format!("line{i}\n")).collect();
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    // Open file at line 80 via CM_OPEN_FILE_FOCUS (simulates LSP goto definition)
    let req = OpenFileRequest::at(dir.path().join("big.txt"), 79, 0);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(2);

    // line80 should be visible on screen (viewport pre-scrolled)
    assert!(
        h.content_contains("line80"),
        "line80 should be visible after goto with pre-scroll"
    );
}
