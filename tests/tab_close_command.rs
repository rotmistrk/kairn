//! Regression: CM_TW_TAB_CLOSE must actually close a clean tab.

mod helpers;

use helpers::{temp_project, TestHarness};
use kairn::commands::{OpenFileRequest, CM_OPEN_FILE_FOCUS};
use txv_widgets::tiled_workspace::commands::CM_TW_TAB_CLOSE;

fn open_file(h: &mut TestHarness, name: &str) {
    let path = h.state.root_dir().join(name);
    h.dispatch_command(CM_OPEN_FILE_FOCUS, Some(Box::new(OpenFileRequest::new(path))));
}

#[test]
fn cm_tw_tab_close_closes_clean_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::new(dir.path());
    open_file(&mut h, "a.rs");
    open_file(&mut h, "b.rs");
    h.run_cycles(2);

    // b.rs should be the active tab and visible
    assert!(h.content_contains("bbb"), "b.rs should be visible");

    // Send CM_TW_TAB_CLOSE to close the active (clean) tab
    h.dispatch_command(CM_TW_TAB_CLOSE, None);
    h.run_cycles(2);

    // b.rs should be gone, a.rs should now be visible
    assert!(h.content_contains("aaa"), "a.rs should be visible after closing b.rs");
    assert!(!h.content_contains("bbb"), "b.rs content should no longer be visible");
}
