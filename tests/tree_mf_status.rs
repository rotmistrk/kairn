mod helpers;
use helpers::{temp_project, TestHarness};

#[test]
fn tree_focus_shows_m_f_in_status() {
    let dir = temp_project(&[("a.txt", "")]);
    let mut h = TestHarness::with_size(dir.path(), 80, 24);
    h.run_cycles(2);
    {
        use kairn::handler::downcast_desktop;
        use kairn::slots::SlotId;
        let desktop = h.program.desktop_mut();
        if let Some(d) = downcast_desktop(desktop) {
            d.focus_panel(SlotId::Left as usize);
        }
    }
    h.run_cycles(3);
    let status = h.row(23);
    assert!(
        status.contains("M-f") || status.contains("ƒ"),
        "M-f should appear when tree focused: {}",
        status
    );
}
