//! Scenario tests for sticky scroll and search-replace.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn alt(ch: char) -> (KeyCode, KeyMod) {
    (
        KeyCode::Char(ch),
        KeyMod {
            alt: true,
            ..KeyMod::default()
        },
    )
}

fn mx_command(h: &mut TestHarness, cmd: &str) {
    h.inject_key(alt('x').0, alt('x').1);
    h.run_cycles(2);
    h.inject_str(cmd);
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);
}

#[test]
fn search_replace_shows_matches() {
    let dir = temp_project(&[
        ("src/a.rs", "fn hello() {}\nfn world() {}\n"),
        ("src/b.rs", "fn hello_again() {}\n"),
    ]);
    let mut h = TestHarness::with_size(dir.path(), 100, 24);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    mx_command(&mut h, "replace /hello/goodbye/");
    // Wait for grep + view
    h.run_cycles(5);

    let screen = h.screen_text();
    assert!(
        screen.contains("goodbye"),
        "should show replacement preview: {}",
        screen
    );
}

#[test]
fn search_replace_applies_change() {
    let dir = temp_project(&[("data.txt", "old_value\nkeep_this\nold_value\n")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 24);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(2);

    mx_command(&mut h, "replace /old_value/new_value/");
    h.run_cycles(5);

    // Apply first match
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // Verify file was changed
    let content = std::fs::read_to_string(dir.path().join("data.txt")).unwrap();
    assert!(
        content.contains("new_value"),
        "file should have replacement: {}",
        content
    );
    assert!(content.contains("keep_this"), "unrelated lines preserved");
}

#[test]
fn yaml_format_basic() {
    // Test the YAML formatter function directly
    let input = "name: hello\nitems:\n- one\n- two\n";
    let result = kairn::format_yaml::format_yaml(input);
    assert!(result.is_ok(), "should parse valid YAML");
    let formatted = result.unwrap();
    assert!(formatted.contains("name:"), "should contain key");
    assert!(formatted.contains("items:"), "should contain items");
}

#[test]
fn yaml_format_rejects_invalid() {
    let input = "{{invalid yaml::: [[";
    let result = kairn::format_yaml::format_yaml(input);
    assert!(result.is_err(), "should reject invalid YAML");
}
