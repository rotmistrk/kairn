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
    eprintln!("SCREEN:\n{}", screen);
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

#[test]
fn powerline_glyphs_in_chrome() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let top = h.row(0);
    // Powerline left cap (U+E0B6) or right cap (U+E0B4)
    assert!(
        top.contains('\u{E0B6}') || top.contains('\u{E0B4}'),
        "chrome should contain Powerline glyphs: {top:?}"
    );
}

#[test]
fn tall_layout_single_chrome_for_bottom_panel() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // In tall mode (80 cols), Shell is at bottom. Count lines containing "Shell"
    let mut shell_lines = 0;
    for y in 0..23 {
        if h.row(y).contains("Shell") {
            shell_lines += 1;
        }
    }
    assert_eq!(
        shell_lines, 1,
        "Shell tab name should appear on exactly one chrome line, got {shell_lines}"
    );
}

#[test]
fn connector_glyphs_at_divider_intersections() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.run_cycles(1);
    let top = h.row(0);
    assert!(top.contains('┬'), "top chrome should have ┬ connector: {top:?}");
}
