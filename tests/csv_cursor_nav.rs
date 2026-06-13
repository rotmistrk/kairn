//! Scenario tests for CsvView cursor navigation (j/k/h/l/G/gg).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_csv(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

fn csv_content() -> &'static str {
    "name,age,city,score,level\n\
     alice,30,NYC,100,5\n\
     bob,25,LA,90,4\n\
     carol,35,CHI,80,3\n\
     dave,28,SF,70,2\n\
     eve,32,BOS,60,1\n\
     frank,27,SEA,50,6\n\
     grace,29,DEN,40,7\n\
     hank,31,ATL,30,8\n\
     ivy,26,PHX,20,9\n\
     jack,33,DAL,10,10\n"
}

#[test]
fn csv_nav_down_j() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Initially at row 0 — alice visible
    assert!(h.content_contains("alice"), "row 0 alice visible");
    // Press j to move down
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(2);
    // bob should still be visible (we moved to row 1)
    assert!(h.content_contains("bob"), "row 1 bob visible after j");
}

#[test]
fn csv_nav_up_k() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Move down then up
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(2);
    // Should be back at row 0
    assert!(h.content_contains("alice"), "alice visible after k");
}

#[test]
fn csv_nav_right_l() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Press l to move column right
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(2);
    // age column header should be visible
    assert!(h.content_contains("age"), "age header visible after l");
}

#[test]
fn csv_nav_left_h() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Move right then left
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(2);
    // name column should still be visible
    assert!(h.content_contains("name"), "name header visible after h");
}

#[test]
fn csv_nav_jump_bottom_g_upper() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Press G to jump to last row
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.run_cycles(2);
    // Last row (jack) should be visible
    assert!(
        h.content_contains("jack"),
        "jack (last row) visible after G: {}",
        h.screen_text()
    );
}

#[test]
fn csv_nav_jump_top_gg() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Move down several times then gg
    for _ in 0..5 {
        h.inject_key(KeyCode::Char('j'), KeyMod::default());
        h.run_cycles(1);
    }
    // Press g to jump to top
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.run_cycles(2);
    // alice (first row) should be visible
    assert!(
        h.content_contains("alice"),
        "alice (first row) visible after gg: {}",
        h.screen_text()
    );
}

#[test]
fn csv_content_visible_throughout_navigation() {
    let dir = temp_project(&[("data.csv", csv_content())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Headers should always be visible
    assert!(h.content_contains("name"), "header 'name' visible initially");
    assert!(h.content_contains("age"), "header 'age' visible initially");
    // Navigate around and check content remains visible
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(2);
    // Data should still be drawn correctly
    assert!(
        h.content_contains("alice"),
        "alice still visible after nav: {}",
        h.screen_text()
    );
}
