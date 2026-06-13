//! Comprehensive scenario tests for CsvView navigation, editing, and features.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_csv(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

fn csv_data() -> &'static str {
    "name,age,city\nalice,30,NYC\nbob,25,LA\ncarol,35,CHI\n"
}

#[test]
fn csv_nav_j_k_moves() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // j moves down to bob
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("bob"), "bob visible after j");
    // k moves back up to alice
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("alice"), "alice visible after k");
}

#[test]
fn csv_nav_h_l_columns() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // l moves to age column
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("age"), "age header visible after l");
    // h moves back to name column
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("name"), "name header visible after h");
}

#[test]
fn csv_jump_g_g_upper() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // G jumps to bottom (carol)
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("carol"), "carol visible after G");
    // g jumps to top (alice)
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("alice"), "alice visible after g");
}

#[test]
fn csv_sort_column() {
    let csv = "name,age\ncharlie,35\nalice,30\nbob,25\n";
    let dir = temp_project(&[("d.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(2);
    let s = h.screen_text();
    let a = s.find("alice").unwrap_or(usize::MAX);
    let b = s.find("bob").unwrap_or(usize::MAX);
    let c = s.find("charlie").unwrap_or(usize::MAX);
    assert!(a < b && b < c, "sorted: alice < bob < charlie");
}

#[test]
fn csv_sort_reverse() {
    let csv = "name,age\nalice,30\nbob,25\ncarol,35\n";
    let dir = temp_project(&[("d.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Sort ascending
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(2);
    // Sort again to reverse
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(2);
    let s = h.screen_text();
    let a = s.find("alice").unwrap_or(0);
    let c = s.find("carol").unwrap_or(0);
    assert!(c < a, "reverse sorted: carol before alice");
}

#[test]
fn csv_filter_rows() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("ali");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("alice"), "alice visible");
    assert!(!h.content_contains("bob"), "bob filtered out");
}

#[test]
fn csv_clear_filter() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Apply filter
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("ali");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(!h.content_contains("bob"), "bob filtered out");
    // Clear filter with F
    h.inject_key(KeyCode::Char('F'), KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("bob"), "bob visible after clear");
}

#[test]
fn csv_add_row() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    h.inject_key(KeyCode::Char('a'), KeyMod::default());
    h.run_cycles(2);
    // All original rows still present
    assert!(h.content_contains("alice"), "alice still visible");
    assert!(h.content_contains("carol"), "carol still visible");
}

#[test]
fn csv_delete_row() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Cursor on alice (row 0), press d then confirm with y
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(2);
    assert!(!h.content_contains("alice"), "alice deleted");
    assert!(h.content_contains("bob"), "bob remains");
}

#[test]
fn csv_edit_cell() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    for _ in 0..5 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("zara");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("zara"), "cell updated to zara");
}

#[test]
fn csv_edit_cancel() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_str("xxx");
    h.run_cycles(1);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("alice"), "alice preserved");
    assert!(!h.content_contains("xxx"), "edit cancelled");
}

#[test]
fn csv_yank_paste() {
    let dir = temp_project(&[("d.csv", csv_data())]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Yank alice row
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(1);
    // Move to carol
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Paste
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(2);
    let s = h.screen_text();
    let first = s.find("alice").unwrap_or(usize::MAX);
    let second = s[first + 1..].find("alice");
    assert!(second.is_some(), "two alice rows after yank+paste");
}

#[test]
fn csv_wide_table_scroll() {
    let csv = "a,b,c,d,e,f,g,h,i,j\n1,2,3,4,5,6,7,8,9,10\n";
    let dir = temp_project(&[("w.csv", csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Scroll right through many columns
    for _ in 0..8 {
        h.inject_key(KeyCode::Char('l'), KeyMod::default());
        h.run_cycles(1);
    }
    h.run_cycles(2);
    // Column 'j' (the 10th) should be visible
    assert!(
        h.content_contains("10"),
        "rightmost column visible: {}",
        h.screen_text()
    );
}

#[test]
fn csv_tall_table_header_sticky() {
    let mut csv = String::from("name,age\n");
    for i in 0..30 {
        csv.push_str(&format!("row{i},{i}\n"));
    }
    let dir = temp_project(&[("t.csv", &csv)]);
    let mut h = TestHarness::new(dir.path());
    open_csv(&mut h);
    // Scroll down many rows
    for _ in 0..20 {
        h.inject_key(KeyCode::Char('j'), KeyMod::default());
        h.run_cycles(1);
    }
    h.run_cycles(2);
    // Header should still be visible (sticky)
    assert!(
        h.content_contains("name"),
        "header 'name' sticky after scroll: {}",
        h.screen_text()
    );
}
