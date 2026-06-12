//! Scenario: opening file at a specific line scrolls it into view.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_core::event::KeyCode;

#[test]
fn open_file_at_line_scrolls_into_view() {
    let content: String = (1..=100).map(|i| format!("line{i}\n")).collect();
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    let req = OpenFileRequest::at(dir.path().join("big.txt"), 50, 0);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(2);

    // line51 (display) should be visible
    assert!(
        h.content_contains("line51"),
        "line51 should be visible after open-at-line-50"
    );
}

#[test]
fn open_file_at_line_not_at_edge() {
    let content: String = (1..=100).map(|i| format!("line{i}\n")).collect();
    let dir = temp_project(&[("big.txt", &content)]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(1);

    let req = OpenFileRequest::at(dir.path().join("big.txt"), 50, 0);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(req)));
    h.run_cycles(2);

    // Lines above and below line51 should also be visible (margin)
    assert!(
        h.content_contains("line49") || h.content_contains("line50"),
        "at least one line before target should be visible"
    );
    assert!(
        h.content_contains("line52") || h.content_contains("line53"),
        "at least one line after target should be visible"
    );
}
