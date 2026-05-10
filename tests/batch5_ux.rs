//! Tests for Batch 5: :e Tab completion, dropdown tabs, name disambiguation.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

// ─── Feature 1: :e Tab completion via project root ─────────────────────────

#[test]
fn tab_completes_filename_from_project_root() {
    let dir = temp_project(&[("Makefile", "all:"), ("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    // Open main.rs from tree, focus editor
    h.inject_key(KeyCode::Down, KeyMod::default()); // skip Makefile, go to main.rs
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Now in editor. Type :e M then Tab
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e M");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    // Command buffer should contain "Makefile"
    assert!(h.contains(":e Makefile"), "Tab should complete 'M' to 'Makefile'");
}

#[test]
fn tab_completes_subdir_file() {
    let dir = temp_project(&[
        ("start.rs", "start"),
        ("src/lib.rs", "lib"),
    ]);
    let mut h = TestHarness::new(dir.path());
    // Open start.rs
    h.inject_key(KeyCode::Down, KeyMod::default()); // past src/ dir
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Type :e src/l then Tab
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e src/l");
    h.inject_key(KeyCode::Tab, KeyMod::default());
    h.run_cycles(1);
    assert!(h.contains(":e src/lib.rs"), "Tab should complete 'src/l' to 'src/lib.rs'");
}

// ─── Feature 2: Dropdown tabs ──────────────────────────────────────────────

#[test]
fn chrome_shows_active_tab_with_count() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    // Open all 3 files
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Chrome should show "c.rs (3)" not "(a.rs)(b.rs)(c.rs)"
    let top = h.row(0);
    assert!(top.contains("❨3❩") || top.contains("3"), "should show tab count: {}", top);
    assert!(!top.contains("(a.rs)"), "should NOT show all tab names: {}", top);
}

#[test]
fn ctrl_shift_down_shows_dropdown_overlay() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // Ctrl-Shift-Down should show dropdown with tab list
    h.inject_key(KeyCode::Down, KeyMod { ctrl: true, alt: false, shift: true });
    h.run_cycles(1);
    let screen = h.screen_text();
    // Dropdown should show numbered entries
    assert!(screen.contains("0:"), "dropdown should show numbered entries: {}", screen);
    assert!(screen.contains("a.rs"), "dropdown should list a.rs");
    assert!(screen.contains("b.rs"), "dropdown should list b.rs");
}

#[test]
fn dropdown_digit_selects_tab() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb"), ("c.rs", "ccc")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // c.rs is active. Open dropdown, press '0' to select a.rs
    h.inject_key(KeyCode::Down, KeyMod { ctrl: true, alt: false, shift: true });
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('0'), KeyMod::default());
    h.run_cycles(1);
    // a.rs should now be active
    assert!(h.contains("aaa"), "pressing '0' should select first tab (a.rs)");
}

#[test]
fn dropdown_esc_cancels() {
    let dir = temp_project(&[("a.rs", "aaa"), ("b.rs", "bbb")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // b.rs is active. Open dropdown, press Esc
    h.inject_key(KeyCode::Down, KeyMod { ctrl: true, alt: false, shift: true });
    h.run_cycles(1);
    h.inject_key(KeyCode::Esc, KeyMod::default());
    h.run_cycles(1);
    // b.rs should still be active, no dropdown visible
    assert!(h.contains("bbb"), "Esc should cancel dropdown, keep b.rs active");
    let screen = h.screen_text();
    assert!(!screen.contains("0:"), "dropdown should be closed after Esc");
}

// ─── Feature 3: Name disambiguation ────────────────────────────────────────

#[test]
fn duplicate_filenames_show_path_suffix() {
    let dir = temp_project(&[
        ("alpha/Cargo.toml", "[alpha]"),
        ("beta/Cargo.toml", "[beta]"),
    ]);
    let mut h = TestHarness::new(dir.path());
    // Open alpha/Cargo.toml
    h.inject_key(KeyCode::Enter, KeyMod::default()); // expand alpha/
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default()); // open Cargo.toml
    h.run_cycles(1);
    // Open beta/Cargo.toml via :e
    h.inject_key(KeyCode::Char(':'), KeyMod::default());
    h.inject_str("e beta/Cargo.toml\n");
    h.run_cycles(1);
    // Active tab should show disambiguated name with path prefix
    let top = h.row(0);
    assert!(top.contains("beta/Cargo.toml"),
        "active tab should show disambiguated path: {}", top);
    // Open dropdown to verify alpha is also disambiguated
    h.inject_key(KeyCode::Down, KeyMod { ctrl: true, alt: false, shift: true });
    h.run_cycles(1);
    let screen = h.screen_text();
    assert!(screen.contains("alpha/Cargo.toml"),
        "dropdown should show disambiguated alpha: {}", screen);
}

#[test]
fn unique_filenames_show_basename_only() {
    let dir = temp_project(&[("foo.rs", "foo"), ("bar.rs", "bar")]);
    let mut h = TestHarness::new(dir.path());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(2), KeyMod::default());
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    // No disambiguation needed — active tab shows basename only
    let top = h.row(0);
    // Active tab is foo.rs (second opened, tree sorts alphabetically: bar, foo)
    assert!(top.contains("foo.rs"), "should show basename: {}", top);
    assert!(!top.contains("/foo.rs"), "should NOT show path for unique names: {}", top);
}
