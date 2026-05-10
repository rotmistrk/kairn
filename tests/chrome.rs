mod helpers;

use helpers::{temp_project, TestHarness};

#[test]
fn top_line_has_box_drawing() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let top = h.row(0);
    assert!(top.contains('─'), "top line missing ─: {}", top);
}

#[test]
fn vertical_dividers_between_slots() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let screen = h.screen_text();
    assert!(screen.contains('│'), "missing vertical divider");
}

#[test]
fn tab_names_in_top_line() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let top = h.row(0);
    assert!(top.contains("Files"), "tab name missing: {}", top);
    // In tall layout (<200 width), Shell tab is in bottom divider, not top
    let screen = h.screen_text();
    assert!(screen.contains("Shell"), "Shell tab missing from screen");
}

#[test]
fn no_outer_borders() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let row1 = h.row(1);
    let first = row1.chars().next().unwrap_or(' ');
    assert_ne!(first, '│', "should not have left outer border");
}
