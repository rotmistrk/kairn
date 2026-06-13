//! Comprehensive scenario tests for StructuredView.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn open_json(h: &mut TestHarness) {
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);
}

#[test]
fn struct_nav_j_k() {
    let dir = temp_project(&[("t.json", r#"{"a":1,"b":2,"c":3}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("a"), "a visible after j/k nav");
    assert!(h.content_contains("b"), "b visible after j/k nav");
    assert!(h.content_contains("c"), "c visible after j/k nav");
}

#[test]
fn struct_jump_g_big_g() {
    let dir = temp_project(&[("t.json", r#"{"a":1,"b":2,"c":3,"d":4,"e":5}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Jump to bottom
    h.inject_key(KeyCode::Char('G'), KeyMod::default());
    h.run_cycles(1);
    // Jump to top
    h.inject_key(KeyCode::Char('g'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("a"), "a visible after g/G jumps");
    assert!(h.content_contains("e"), "e visible after g/G jumps");
}

#[test]
fn struct_expand_collapse() {
    let dir = temp_project(&[("t.json", r#"{"obj":{"x":1,"y":2},"z":3}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    assert!(h.content_contains("x"), "x visible when expanded");
    // Navigate to "obj" and collapse
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.content_contains("x"), "x hidden after collapse");
    // Expand with l
    h.inject_key(KeyCode::Char('l'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("x"), "x visible after expand with l");
}

#[test]
fn struct_edit_value_enter_commit() {
    let dir = temp_project(&[("t.json", r#"{"val":"old"}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    for _ in 0..3 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("new");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("new"), "value committed as 'new'");
}

#[test]
fn struct_edit_esc_cancels() {
    let dir = temp_project(&[("t.json", r#"{"val":"keep"}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_str("NOPE");
    h.run_cycles(1);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("keep"), "original preserved after Esc");
    assert!(!h.content_contains("NOPE"), "typed text discarded after Esc");
}

#[test]
fn struct_filter_shows_matching() {
    let dir = temp_project(&[("t.json", r#"{"alpha":1,"beta":2,"gamma":3}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    h.inject_str("beta");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
    assert!(h.content_contains("beta"), "beta visible with filter");
    assert!(!h.content_contains("gamma"), "gamma hidden by filter");
}

#[test]
fn struct_sort_children() {
    let dir = temp_project(&[("t.json", r#"{"c":3,"a":1,"b":2}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(1);
    let screen = h.screen_text();
    let pos_a = screen.find("a");
    let pos_b = screen.find("b");
    let pos_c = screen.find("c");
    assert!(pos_a < pos_b, "a before b after sort");
    assert!(pos_b < pos_c, "b before c after sort");
}

#[test]
fn struct_add_sibling() {
    let dir = temp_project(&[("t.json", r#"[1,2]"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("null"), "new sibling null added");
}

#[test]
fn struct_add_child() {
    let dir = temp_project(&[("t.json", r#"{"obj":[1,2]}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Navigate to "obj" array container
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Press 'b' to add child
    h.inject_key(KeyCode::Char('b'), KeyMod::default());
    h.run_cycles(2);
    // Array should now have 3 elements
    assert!(
        h.content_contains("[3]"),
        "array should have 3 elements after add child"
    );
}

#[test]
fn struct_delete_node() {
    let dir = temp_project(&[("t.json", r#"{"a":1,"b":2}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("b"), "b still present");
    // Only one key should remain — check root shows {1}
    assert!(h.content_contains("{1}"), "root shows {{1}} after delete");
}

#[test]
fn struct_undo_restores() {
    let dir = temp_project(&[("t.json", r#"{"a":1,"b":2}"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.content_contains("├─a"), "a gone after delete");
    h.inject_key(KeyCode::Char('u'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("a"), "a restored after undo");
}

#[test]
fn struct_yank_paste() {
    let dir = temp_project(&[("t.json", r#"[10,20]"#)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Yank first element
    h.inject_key(KeyCode::Char('y'), KeyMod::default());
    h.run_cycles(1);
    // Paste — should add a duplicate
    h.inject_key(KeyCode::Char('p'), KeyMod::default());
    h.run_cycles(1);
    assert!(h.content_contains("[3]"), "array has 3 elements after yank+paste");
}

#[test]
fn struct_deep_nesting() {
    let json = r#"{"l1":{"l2":{"l3":{"l4":{"l5":"deep"}}}}}"#;
    let dir = temp_project(&[("t.json", json)]);
    let mut h = TestHarness::new(dir.path());
    open_json(&mut h);
    // Navigate down expanding all levels
    for _ in 0..5 {
        h.inject_key(KeyCode::Char('j'), KeyMod::default());
        h.run_cycles(1);
    }
    assert!(h.content_contains("deep"), "deep value visible after expanding");
    // Collapse from inside
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(1);
    assert!(!h.content_contains("deep"), "deep hidden after collapsing");
}
