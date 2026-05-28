//! Tests for status bar background color consistency.
//! Verifies that all cells on the status bar have the correct background color,
//! with no black (Color::Reset) gaps.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::cell::Color;
use txv_core::event::{KeyCode, KeyMod};
use txv_core::palette::{palette, StyleId};

/// Helper: get the expected status bar background color.
fn bar_bg() -> Color {
    palette().style(StyleId::StatusBar).bg
}

/// Helper: get the expected modal background color.
fn modal_bg() -> Color {
    palette().style(StyleId::StatusBarModal).bg
}

/// Status bar row is the last row.
fn status_row(h: &TestHarness) -> u16 {
    let surface = h.backend.surface().expect("no surface");
    surface.height() - 1
}

/// Every cell on the status bar row must have a non-black background when dormant.
/// Bug: gap between M-x label and next item had Color::Reset (black) bg.
#[test]
fn status_bar_dormant_no_black_bg() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    let surface = h.backend.surface().expect("no surface");
    let w = surface.width();
    let y = surface.height() - 1;
    let expected_bg = bar_bg();

    for x in 0..w {
        let cell = surface.cell(x, y);
        assert!(
            cell.style.bg != Color::Reset && cell.style.bg != Color::Ansi(0),
            "cell at x={x} has black bg ({:?}), expected bar bg ({expected_bg:?}). char={:?}",
            cell.style.bg,
            cell.ch,
        );
    }
}

/// When M-x modal is active, the right power cap must have fg=modal_bg, bg=bar_bg.
/// Bug: right cap was black-on-black (both fg and bg were Color::Reset).
#[test]
fn status_bar_active_modal_right_cap_colors() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Activate M-x modal
    h.inject_key(
        KeyCode::Char('x'),
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(2);

    let surface = h.backend.surface().expect("no surface");
    let w = surface.width();
    let y = surface.height() - 1;
    let expected_bar_bg = bar_bg();
    let expected_modal_bg = modal_bg();

    // Find the right power cap (U+E0B4) on status bar row
    let mut found_cap = false;
    for x in 0..w {
        let cell = surface.cell(x, y);
        if cell.ch == '\u{e0b4}' {
            found_cap = true;
            assert_eq!(
                cell.style.fg, expected_modal_bg,
                "right cap at x={x}: fg should be modal_bg ({expected_modal_bg:?}), got {:?}",
                cell.style.fg,
            );
            assert_eq!(
                cell.style.bg, expected_bar_bg,
                "right cap at x={x}: bg should be bar_bg ({expected_bar_bg:?}), got {:?}",
                cell.style.bg,
            );
            break;
        }
    }
    assert!(found_cap, "right power cap (U+E0B4) not found on status bar");
}

/// When M-x modal is active, cells after the right cap must have bar bg, not black.
/// Bug: area after right power cap was black until the next item.
#[test]
fn status_bar_active_modal_no_black_after_cap() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Activate M-x modal
    h.inject_key(
        KeyCode::Char('x'),
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(2);

    let surface = h.backend.surface().expect("no surface");
    let w = surface.width();
    let y = surface.height() - 1;
    let expected_bar_bg = bar_bg();

    // Find the right power cap, then check all cells after it
    let mut cap_x = None;
    for x in 0..w {
        let cell = surface.cell(x, y);
        if cell.ch == '\u{e0b4}' {
            cap_x = Some(x);
            break;
        }
    }
    let cap_x = cap_x.expect("right power cap not found");

    for x in (cap_x + 1)..w {
        let cell = surface.cell(x, y);
        assert!(
            cell.style.bg != Color::Reset && cell.style.bg != Color::Ansi(0),
            "cell at x={x} (after right cap at {cap_x}) has black bg ({:?}), expected bar bg ({expected_bar_bg:?}). char={:?}",
            cell.style.bg,
            cell.ch,
        );
    }
}

/// Clock must appear in status bar at reasonable terminal widths.
#[test]
fn clock_visible_at_120_cols() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    // Create a .git/HEAD to simulate a branch
    std::fs::create_dir(dir.path().join(".git")).unwrap();
    std::fs::write(
        dir.path().join(".git/HEAD"),
        "ref: refs/heads/feature/long-branch-name\n",
    )
    .unwrap();
    let mut h = TestHarness::with_size(dir.path(), 120, 24);
    h.run_cycles(2);
    let screen = h.screen_text();
    let has_time = regex::Regex::new(r"\d\d:\d\d").unwrap().is_match(&screen);
    assert!(
        has_time,
        "clock should be visible at 120 cols. Status bar: {}",
        h.row(23)
    );
}

/// After executing a command via M-x, reopening M-x should show old text selected.
/// Typing should replace it.
#[test]
fn mx_reopen_selects_old_text_typing_replaces() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open M-x, type "help", press Enter
    h.inject_key(
        KeyCode::Char('x'),
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(1);
    h.inject_str("help");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // Reopen M-x — old text "help" should be there but selected
    h.inject_key(
        KeyCode::Char('x'),
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(2);

    // Type "e" — should replace "help" with "e"
    h.inject_key(KeyCode::Char('e'), KeyMod::default());
    h.run_cycles(1);

    let row = h.row(23);
    assert!(
        row.contains("e"),
        "after typing 'e', should see 'e' in prompt, got: {row}"
    );
    assert!(!row.contains("help"), "old text 'help' should be replaced, got: {row}");
}

/// After reopening M-x, pressing Left should deselect and allow editing.
#[test]
fn mx_reopen_nav_deselects() {
    let dir = temp_project(&[("main.rs", "fn main() {}")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);

    // Open M-x, type "test", press Enter
    h.inject_key(
        KeyCode::Char('x'),
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(1);
    h.inject_str("test");
    h.run_cycles(1);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(2);

    // Reopen M-x
    h.inject_key(
        KeyCode::Char('x'),
        KeyMod {
            ctrl: false,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(2);

    // Press Left to deselect, then type "X"
    h.inject_key(KeyCode::Left, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('X'), KeyMod::default());
    h.run_cycles(1);

    let row = h.row(23);
    // "test" should still be there with "X" inserted
    assert!(
        row.contains("tesX") || row.contains("teXt") || row.contains("tXst") || row.contains("Xest"),
        "after Left+X, should have X inserted into 'test', got: {row}"
    );
}
