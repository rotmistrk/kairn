mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn open_json_shows_structured_view() {
    let json = r#"{"name":"test","count":42}"#;
    let dir = temp_project(&[("data.json", json)]);
    let mut h = TestHarness::new(dir.path());
    // Open the file (first in tree)
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Should show structured view with key names visible
    assert!(h.content_contains("name"), "should show 'name' key");
    assert!(h.content_contains("count"), "should show 'count' key");
}

#[test]
fn navigate_up_down() {
    let json = r#"{"a":1,"b":2,"c":3}"#;
    let dir = temp_project(&[("nav.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Focus center panel
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Move down twice
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Move up once
    h.inject_key(KeyCode::Char('k'), KeyMod::default());
    h.run_cycles(1);
    // View should still render correctly (no crash)
    assert!(h.content_contains("a"), "should still show keys after navigation");
}

#[test]
fn expand_collapse() {
    let json = r#"{"obj":{"x":1,"y":2},"z":3}"#;
    let dir = temp_project(&[("nested.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Initially expanded — children should be visible
    assert!(
        h.content_contains("x"),
        "nested key 'x' should be visible when expanded"
    );
    // Navigate to the "obj" container and collapse it (j then h to collapse)
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('h'), KeyMod::default());
    h.run_cycles(1);
    // After collapse, children should be hidden
    assert!(
        !h.content_contains("x"),
        "nested key 'x' should be hidden after collapse"
    );
}

#[test]
fn invalid_json_falls_back_to_editor() {
    let dir = temp_project(&[("bad.json", "{ not valid json !!!")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Should fall back to editor — show raw content
    assert!(
        h.content_contains("not valid json"),
        "invalid JSON should open in editor showing raw text"
    );
}

#[test]
fn edit_value() {
    let json = r#"{"name":"old"}"#;
    let dir = temp_project(&[("edit.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "name" scalar (j to move to child)
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Tab to Value column
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Enter to start editing
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Clear existing text and type new value
    // The editor starts with "old" selected, cursor at end
    // Use Home then select-all via repeated Delete, then type new
    h.inject_key(KeyCode::Home, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Delete, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Delete, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Delete, KeyMod::default());
    h.run_cycles(1);
    // Type "new"
    h.inject_str("new");
    h.run_cycles(1);
    // Commit with Enter
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Value should now show "new"
    assert!(h.content_contains("new"), "value should be updated to 'new'");
}

#[test]
fn add_sibling_array() {
    let json = r#"[1,2,3]"#;
    let dir = temp_project(&[("arr.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to first array element
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Press 'n' to add sibling — in array, adds null silently
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(1);
    // Should now show "null" as a new element
    assert!(h.content_contains("null"), "new null element should appear in array");
}

#[test]
fn add_sibling_dict() {
    let json = r#"{"a":1}"#;
    let dir = temp_project(&[("dict.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "a" entry
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Press 'n' — in dict, should start key edit (InlineEditor active)
    h.inject_key(KeyCode::Char('n'), KeyMod::default());
    h.run_cycles(1);
    // Type a key name and commit
    h.inject_str("newkey");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Should show the new key
    assert!(h.content_contains("newkey"), "new dict key should appear");
}

#[test]
fn clone_in_array() {
    let json = r#"[10,20]"#;
    let dir = temp_project(&[("clone.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to first element
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Clone it
    h.inject_key(KeyCode::Char('c'), KeyMod::default());
    h.run_cycles(1);
    // Should now show [3] in the root (was [2])
    assert!(
        h.content_contains("[3]"),
        "array should now have 3 elements after clone"
    );
}

#[test]
fn delete_node() {
    let json = r#"{"a":1,"b":2,"c":3}"#;
    let dir = temp_project(&[("del.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to "a"
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Delete it
    h.inject_key(KeyCode::Char('d'), KeyMod::default());
    h.run_cycles(1);
    // "a" should be gone
    assert!(!h.content_contains("├─a"), "key 'a' should be removed after delete");
    // "b" and "c" should remain
    assert!(h.content_contains("b"), "key 'b' should still exist");
    assert!(h.content_contains("c"), "key 'c' should still exist");
}

#[test]
fn swap_nodes() {
    let json = r#"[1,2,3]"#;
    let dir = temp_project(&[("swap.json", json)]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);
    // Navigate to first element [0]
    h.inject_key(KeyCode::Char('j'), KeyMod::default());
    h.run_cycles(1);
    // Swap down — element with value "1" should move to position [1]
    h.inject_key(KeyCode::Char('J'), KeyMod::default());
    h.run_cycles(1);
    // Now swap up — should move back to [0]
    h.inject_key(KeyCode::Char('K'), KeyMod::default());
    h.run_cycles(1);
    // View should still render correctly (no crash, values still present)
    assert!(h.content_contains("1"), "value 1 should still be visible");
    assert!(h.content_contains("2"), "value 2 should still be visible");
    assert!(h.content_contains("3"), "value 3 should still be visible");
}
