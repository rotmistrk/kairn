mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn sort_dict_by_key() {
    let json = r#"{"b":2,"a":1,"c":3}"#;
    let dir = temp_project(&[("sort.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Cursor is on root dict — press 's' to sort ascending
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(1);
    // Verify serialized order: a, b, c
    // The display should show keys in sorted order
    let screen = h.screen_text();
    let pos_a = screen.find("├─a").or_else(|| screen.find("a"));
    let pos_b = screen.find("├─b").or_else(|| screen.find("b"));
    let pos_c = screen.find("└─c").or_else(|| screen.find("c"));
    assert!(pos_a < pos_b, "a should come before b after sort");
    assert!(pos_b < pos_c, "b should come before c after sort");
}

#[test]
fn sort_array_numeric() {
    let json = r#"[3,1,2]"#;
    let dir = temp_project(&[("nums.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Cursor on root array — sort ascending
    h.inject_key(KeyCode::Char('s'), KeyMod::default());
    h.run_cycles(1);
    // After sort, values should be in order 1, 2, 3
    // Check row-by-row: row with [0] should have value 1
    let screen = h.screen_text();
    // Find lines containing array indices
    let lines: Vec<&str> = screen.lines().collect();
    let idx_lines: Vec<&&str> = lines
        .iter()
        .filter(|l| l.contains("[0]") || l.contains("[1]") || l.contains("[2]"))
        .collect();
    assert!(idx_lines.len() >= 3, "should have 3 array element lines");
    assert!(idx_lines[0].contains("│1"), "[0] should have value 1 after sort");
    assert!(idx_lines[1].contains("│2"), "[1] should have value 2 after sort");
    assert!(idx_lines[2].contains("│3"), "[2] should have value 3 after sort");
}

#[test]
fn sort_by_path() {
    let json = r#"[{"name":"banana"},{"name":"apple"}]"#;
    let dir = temp_project(&[("objs.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Press 'S' to start sort-by-path
    h.inject_key(KeyCode::Char('S'), KeyMod::default());
    h.run_cycles(1);
    // Clear the default "." and type "name"
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Delete, KeyMod::default());
    h.run_cycles(1);
    h.inject_str("name");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // After sort by .name, "apple" should come before "banana"
    let screen = h.screen_text();
    let pos_apple = screen.find("apple");
    let pos_banana = screen.find("banana");
    assert!(pos_apple.is_some(), "should show apple");
    assert!(pos_banana.is_some(), "should show banana");
    assert!(
        pos_apple < pos_banana,
        "apple should come before banana after sort by .name"
    );
}

#[test]
fn filter_shows_matching() {
    let json = r#"{"alpha":1,"beta":2,"gamma":3}"#;
    let dir = temp_project(&[("filt.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Press 'f' to start filter
    h.inject_key(KeyCode::Char('f'), KeyMod::default());
    h.run_cycles(1);
    // Type "beta"
    h.inject_str("beta");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // "beta" should be visible, "gamma" should not
    assert!(h.content_contains("beta"), "filtered key 'beta' should be visible");
    assert!(
        !h.content_contains("gamma"),
        "non-matching key 'gamma' should be hidden"
    );
    // Clear filter with 'F'
    h.inject_key(KeyCode::Char('F'), KeyMod::default());
    h.run_cycles(1);
    assert!(
        h.content_contains("gamma"),
        "gamma should reappear after clearing filter"
    );
}

#[test]
fn save_writes_file() {
    let json = r#"{"val":"original"}"#;
    let dir = temp_project(&[("save.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "val" scalar
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value column
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Edit value
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    // Delete "original" (8 chars)
    for _ in 0..8 {
        h.inject_key(KeyCode::Delete, KeyMod::default());
        h.run_cycles(1);
    }
    h.inject_str("changed");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Verify edit took effect
    assert!(h.content_contains("changed"), "value should show 'changed' after edit");
    // Save via CM_SAVE dispatched through the group
    let event = txv_core::event::Event::Command {
        id: kairn::commands::CM_SAVE,
        data: None,
    };
    h.backend.inject(event);
    h.run_cycles(1);
    // Read file and verify
    let content = std::fs::read_to_string(dir.path().join("save.json")).unwrap();
    assert!(content.contains("changed"), "saved file should contain 'changed'");
}
