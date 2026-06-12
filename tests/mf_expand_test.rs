//! Test: M-f expands to show file operations when pressed.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn mf_expands_to_show_file_operations() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    // Focus tree
    {
        use kairn::handler::downcast_desktop;
        use kairn::slots::SlotId;
        let desktop = h.program.desktop_mut();
        if let Some(d) = downcast_desktop(desktop) {
            d.focus_panel(SlotId::Left as usize);
        }
    }
    h.run_cycles(3);
    let status_before = h.row(23);
    assert!(status_before.contains("M-f"), "M-f hint should show: {}", status_before);

    // Press Alt-f to activate the M-f modal
    h.inject_key(KeyCode::Char('f'), KeyMod::ALT);
    h.run_cycles(2);

    let status_after = h.row(23);
    // Should show expanded file operations (new, dir, del, ren, etc.)
    assert!(
        status_after.contains("new") || status_after.contains("del") || status_after.contains("ren"),
        "M-f should expand to show file ops: {}",
        status_after
    );
}
