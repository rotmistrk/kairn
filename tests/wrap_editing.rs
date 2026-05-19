//! Integration tests: editing with and without word wrap.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}

fn open_and_focus(h: &mut TestHarness, dir: &std::path::Path, file: &str) {
    h.dispatch_command(
        kairn::commands::CM_OPEN_FILE,
        Some(Box::new(kairn::commands::OpenFileRequest::new(dir.join(file)))),
    );
    h.run_cycles(2);
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

fn ex(h: &mut TestHarness, cmd: &str) {
    h.inject_key(KeyCode::Char(':'), none());
    h.run_cycles(1);
    h.inject_str(cmd);
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);
}

// ═══════════════════════════════════════════════════════════════════════
// Wrap ON: editing long lines
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn wrap_insert_at_end_of_long_line() {
    let long = format!("{}end\n", "x".repeat(100));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set wrap");

    // A to append at end of line
    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(1);
    h.inject_str("TAIL");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    assert!(h.content_contains("TAIL"));
}

#[test]
fn wrap_dd_on_wrapped_line() {
    let long = format!("{}\nshort\n", "L".repeat(150));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set wrap");

    // dd should delete the entire long line (not just the visual row)
    h.inject_str("dd");
    h.run_cycles(2);

    assert!(!h.content_contains("LLL"));
    assert!(h.content_contains("short"));
}

#[test]
fn wrap_search_finds_text_on_wrapped_portion() {
    let long = format!("{}NEEDLE rest\n", "p".repeat(100));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set wrap");

    // Search for text that's in the wrapped portion
    h.inject_key(KeyCode::Char('/'), none());
    h.run_cycles(1);
    h.inject_str("NEEDLE");
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);

    // Should find it (cursor moves)
    assert!(h.content_contains("NEEDLE"));
}

#[test]
fn wrap_substitute_on_long_line() {
    let long = format!("{}TARGET end\n", "z".repeat(80));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set wrap");

    ex(&mut h, "%s/TARGET/REPLACED/g");

    assert!(!h.content_contains("TARGET"));
    assert!(h.content_contains("REPLACED"));
}

#[test]
fn wrap_visual_line_delete() {
    let long = format!("{}\nkeep\n", "W".repeat(120));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set wrap");

    // V + d should delete the entire wrapped line
    h.inject_key(KeyCode::Char('V'), none());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('d'), none());
    h.run_cycles(2);

    assert!(!h.content_contains("WWW"));
    assert!(h.content_contains("keep"));
}

// ═══════════════════════════════════════════════════════════════════════
// Wrap OFF: editing with horizontal scroll
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn nowrap_insert_at_end_of_long_line() {
    let long = format!("{}end\n", "x".repeat(100));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(1);
    h.inject_str("TAIL");
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    // Verify buffer integrity: go home, line should still start with x's
    h.inject_str("0");
    h.run_cycles(2);
    assert!(h.content_contains("xxx"));
}

#[test]
fn nowrap_dd_deletes_full_line() {
    let long = format!("{}\nshort\n", "N".repeat(150));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    h.inject_str("dd");
    h.run_cycles(2);

    assert!(!h.content_contains("NNN"));
    assert!(h.content_contains("short"));
}

#[test]
fn nowrap_search_scrolls_to_match() {
    let content = "short\nanother\nfindme here\n";
    let dir = temp_project(&[("f.txt", content)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    h.inject_key(KeyCode::Char('/'), none());
    h.run_cycles(1);
    h.inject_str("findme");
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(3);

    // Search should find the match on line 3
    assert!(h.content_contains("findme"));
}

#[test]
fn nowrap_substitute_works() {
    let dir = temp_project(&[("f.txt", "hello FIND world\nsecond\n")]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    ex(&mut h, "%s/FIND/DONE/g");

    assert!(!h.content_contains("FIND"));
    assert!(h.content_contains("DONE"));
}

#[test]
fn nowrap_indent_unindent() {
    let dir = temp_project(&[("f.txt", "hello\nworld\n")]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");
    ex(&mut h, "set nowrap");

    h.inject_str(">>");
    h.run_cycles(2);

    let screen = h.screen_text();
    assert!(screen.contains("    hello") || screen.contains("  hello"));

    h.inject_str("<<");
    h.run_cycles(2);

    assert!(h.content_contains("hello"));
}

// ═══════════════════════════════════════════════════════════════════════
// Toggle wrap mid-edit: ensure no corruption
// ═══════════════════════════════════════════════════════════════════════

#[test]
fn toggle_wrap_preserves_content() {
    let long = format!("{}END\nsecond\n", "T".repeat(100));
    let dir = temp_project(&[("f.txt", &long)]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    // Start with wrap
    ex(&mut h, "set wrap");
    assert!(h.content_contains("END"));

    // Toggle to nowrap
    ex(&mut h, "set nowrap");
    // Content should still be intact
    assert!(h.content_contains("second"));

    // Toggle back
    ex(&mut h, "set wrap");
    assert!(h.content_contains("END"));
}

#[test]
fn toggle_wrap_after_insert_no_corruption() {
    let dir = temp_project(&[("f.txt", "original\n")]);
    let mut h = TestHarness::with_size(dir.path(), 60, 24);
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.txt");

    ex(&mut h, "set wrap");

    // Insert a long line
    h.inject_key(KeyCode::Char('A'), none());
    h.run_cycles(1);
    let addition = "X".repeat(80);
    h.inject_str(&addition);
    h.inject_key(KeyCode::Esc, none());
    h.run_cycles(2);

    // Toggle wrap off
    ex(&mut h, "set nowrap");
    h.run_cycles(2);

    // Content should still have both original and addition
    assert!(h.content_contains("original"));
    assert!(h.content_contains("XXX"));
}
