//! Help quality tests — ensure help key bindings have meaningful descriptions.
//!
//! Tests that all key bindings shown in :help keys have human-readable
//! descriptions, not raw "cmd:NNN" identifiers.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn get_help_content(h: &TestHarness) -> String {
    h.screen_text()
}

fn navigate_to_topic(h: &mut TestHarness, topic: &str) {
    h.inject_key(KeyCode::Char('x'), KeyMod::ALT);
    h.run_cycles(2);
    h.inject_str(&format!("help {topic}"));
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(3);
}

// =============================================================================
// TEST: No "cmd:NNN" entries in help
// =============================================================================

#[test]
fn help_keys_has_no_raw_command_ids() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());

    navigate_to_topic(&mut h, "keys");
    let content = get_help_content(&h);

    // Check for "cmd:" pattern which indicates a missing label
    let cmd_lines: Vec<&str> = content.lines().filter(|l| l.contains("cmd:")).collect();
    assert!(
        cmd_lines.is_empty(),
        "Help should not contain raw 'cmd:NNN' entries. Found {} occurrences:\n{}",
        cmd_lines.len(),
        cmd_lines.join("\n")
    );
}

// =============================================================================
// TEST: All key bindings have meaningful (>3 char) descriptions
// =============================================================================

#[test]
fn help_keys_have_meaningful_descriptions() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());

    navigate_to_topic(&mut h, "keys");
    let content = get_help_content(&h);

    // Extract lines that look like key bindings (start with spaces, have a key pattern)
    let binding_lines: Vec<&str> = content
        .lines()
        .filter(|l| {
            let trimmed = l.trim_start();
            !trimmed.is_empty()
                && !trimmed.starts_with('─')
                && !trimmed.starts_with('→')
                && !trimmed.starts_with(':')
                && l.starts_with("  ")
        })
        .collect();

    let mut bad_entries = Vec::new();
    for line in &binding_lines {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let desc = parts[1..].join(" ");
            if desc.len() < 4 {
                bad_entries.push(format!("{} -> '{}' (too short)", line.trim(), desc));
            }
        }
    }

    assert!(
        bad_entries.is_empty(),
        "Some key bindings have descriptions that are too short:\n{}",
        bad_entries.join("\n")
    );
}

// =============================================================================
// TEST: Tab navigation keys have proper descriptions (not cmd:NNN)
// =============================================================================

#[test]
fn help_documents_tab_navigation() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());

    navigate_to_topic(&mut h, "keys");
    let content = get_help_content(&h);

    // If any tab navigation keys are present, they must have proper descriptions
    let tab_patterns = ["M-;", "M-'", "M-w", "M-0", "Alt-;", "Alt-'", "Alt-w", "Alt-0"];

    for pattern in tab_patterns {
        for line in content.lines() {
            if line.contains(pattern) {
                assert!(
                    !line.contains("cmd:"),
                    "Tab key '{}' has raw command ID: {}",
                    pattern,
                    line
                );
            }
        }
    }
}

// =============================================================================
// TEST: Panel resize keys have proper descriptions (not cmd:NNN)
// =============================================================================

#[test]
fn help_documents_resize_keys() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());

    navigate_to_topic(&mut h, "keys");
    let content = get_help_content(&h);

    // Resize-related lines should not have raw command IDs
    let resize_indicators = ["resize", "grow", "shrink", "panel"];

    for line in content.lines() {
        let line_lower = line.to_lowercase();
        if resize_indicators.iter().any(|ind| line_lower.contains(ind)) {
            assert!(
                !line.contains("cmd:"),
                "Resize documentation has raw command ID: {}",
                line
            );
        }
    }
}

// =============================================================================
// TEST: Editor help has no raw command IDs
// =============================================================================

#[test]
fn help_editor_topic_has_no_cmd_ids() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());

    navigate_to_topic(&mut h, "editor");
    let content = get_help_content(&h);

    assert!(
        !content.contains("cmd:"),
        "Editor help should not contain raw command IDs"
    );
}

// =============================================================================
// TEST: Tree help has no raw command IDs
// =============================================================================

#[test]
fn help_tree_topic_has_no_cmd_ids() {
    let dir = temp_project(&[("dummy.txt", "")]);
    let mut h = TestHarness::new(dir.path());

    navigate_to_topic(&mut h, "tree");
    let content = get_help_content(&h);

    assert!(
        !content.contains("cmd:"),
        "Tree help should not contain raw command IDs"
    );
}
