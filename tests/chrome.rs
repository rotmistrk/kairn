mod helpers;

use helpers::{run_and_capture, setup, temp_project};

#[test]
fn top_line_has_box_drawing() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    let top = be.row(0);
    assert!(top.contains('─'), "top line missing ─: {}", top);
}

#[test]
fn vertical_dividers_between_slots() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    let screen = be.screen_text();
    assert!(screen.contains('│'), "missing vertical divider");
}

#[test]
fn tab_names_in_top_line() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    let top = be.row(0);
    assert!(top.contains("(Files)"), "tab name missing: {}", top);
    assert!(top.contains("(Shell)"), "tab name missing: {}", top);
}

#[test]
fn no_outer_borders() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    // Left edge of row 1 should not be a border char
    let row1 = be.row(1);
    let first = row1.chars().next().unwrap_or(' ');
    assert_ne!(first, '│', "should not have left outer border");
}
