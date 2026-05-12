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
