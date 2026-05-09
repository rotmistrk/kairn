mod helpers;

use helpers::{temp_project, TestHarness};

#[test]
fn status_bar_shows_key_hints() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let last_row = h.row(23);
    assert!(last_row.contains("F1:Help"));
    assert!(last_row.contains("^Q:Quit"));
}

#[test]
fn status_bar_shows_mx_hint() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    let last_row = h.row(23);
    assert!(last_row.contains("M-x"));
}
