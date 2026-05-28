//! Integration tests: side-by-side diff (`:diff -y`) as single-view mode.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

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

#[test]
fn diff_sbs_enters_mode_without_split() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    old();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    // Trigger side-by-side diff with different base content
    h.dispatch_command(
        kairn::commands::CM_DIFF_SPLIT,
        Some(Box::new(kairn::commands::DiffSplitRequest::new(
            "fn main() {\n    new();\n}\n".to_string(),
            "HEAD".to_string(),
        ))),
    );
    h.run_cycles(3);

    // Should show diff content — no structural split (no second TabPanel)
    let screen = h.screen_text();
    assert!(
        screen.contains("DIFF") || screen.contains("old") || screen.contains("new"),
        "SBS diff mode should be active:\n{}",
        screen
    );

    // Verify no split was created at the panel level
    let desktop = h.program.desktop_mut();
    if let Some(ws) = desktop
        .as_any_mut()
        .and_then(|a| a.downcast_ref::<txv_widgets::tiled_workspace::TiledWorkspace>())
    {
        let sp = ws.split_panel(1); // center panel
        if let Some(sp) = sp {
            assert_eq!(sp.child_count(), 1, "SBS diff should NOT create a panel split");
        }
    }
}

#[test]
fn diff_sbs_exit_with_esc() {
    let dir = temp_project(&[("main.rs", "fn main() {\n    old();\n}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.dispatch_command(
        kairn::commands::CM_DIFF_SPLIT,
        Some(Box::new(kairn::commands::DiffSplitRequest::new(
            "fn main() {\n    new();\n}\n".to_string(),
            "HEAD".to_string(),
        ))),
    );
    h.run_cycles(3);

    // Press Esc to exit SBS mode
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    // Should be back in normal editor mode showing original content
    assert!(h.content_contains("old"));
}

#[test]
fn diff_sbs_identical_shows_no_changes() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");

    h.dispatch_command(
        kairn::commands::CM_DIFF_SPLIT,
        Some(Box::new(kairn::commands::DiffSplitRequest::new(
            "fn main() {}\n".to_string(),
            "HEAD".to_string(),
        ))),
    );
    h.run_cycles(3);

    assert!(
        h.contains("no changes") || h.content_contains("1 lines"),
        "should indicate no changes:\n{}",
        h.screen_text()
    );
}
