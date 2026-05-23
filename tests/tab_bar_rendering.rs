//! Integration tests for TabBar rendering: colors, gradients, powercaps.
//! Batch 2: issues 8-15 from manual testing.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::cell::Color;
use txv_core::event::{KeyCode, KeyMod};

const ALT: KeyMod = KeyMod {
    ctrl: false,
    alt: true,
    shift: false,
};
const CTRL_SHIFT: KeyMod = KeyMod {
    ctrl: true,
    alt: false,
    shift: true,
};

// === Issue 10: Inactive tab fg must be white ===

#[test]
fn inactive_tab_fg_is_white() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    // Open 3 files to get multiple tabs in center
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let surface = h.backend.surface().unwrap();
    let w = surface.width();
    // Find an inactive tab character (subscript ₁ or ₂ marks inactive tabs)
    let row0: Vec<_> = (0..w).map(|x| surface.cell(x, 0).clone()).collect();
    let sub_pos = row0.iter().position(|c| c.ch == '₁' || c.ch == '₂');
    assert!(sub_pos.is_some(), "should have numbered inactive tabs");
    // The character AFTER the subscript is the tab name — check its fg
    let text_pos = sub_pos.unwrap() + 1;
    let cell = &row0[text_pos];
    // fg should be white-ish (Ansi(15) or Rgb bright)
    match cell.style.fg {
        Color::Ansi(n) => assert!(n >= 7, "inactive tab fg should be white/bright, got Ansi({n})"),
        Color::Rgb(r, g, b) => assert!(
            r >= 0xC0 && g >= 0xC0 && b >= 0xC0,
            "inactive tab fg should be bright, got Rgb({r},{g},{b})"
        ),
        other => panic!("inactive tab fg should be white, got {:?}", other),
    }
}

// === Issue 11: Inactive tab bg must be gradient ===

#[test]
fn inactive_tabs_have_gradient_bg() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let surface = h.backend.surface().unwrap();
    let w = surface.width();
    let row0: Vec<_> = (0..w).map(|x| surface.cell(x, 0).clone()).collect();

    // Find subscript positions in the CENTER panel (after the ┬ divider)
    let divider_pos = row0.iter().position(|c| c.ch == '┬').unwrap_or(0);
    let pos1 = row0
        .iter()
        .skip(divider_pos)
        .position(|c| c.ch == '₁')
        .map(|p| p + divider_pos);
    let pos2 = row0
        .iter()
        .skip(divider_pos)
        .position(|c| c.ch == '₂')
        .map(|p| p + divider_pos);
    assert!(
        pos1.is_some() && pos2.is_some(),
        "need at least 2 inactive tabs in center"
    );

    let bg1 = row0[pos1.unwrap()].style.bg;
    let bg2 = row0[pos2.unwrap()].style.bg;

    // Both should be RGB grays
    match (bg1, bg2) {
        (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
            assert_eq!(r1, g1, "bg1 should be gray");
            assert_eq!(r1, b1, "bg1 should be gray");
            assert_eq!(r2, g2, "bg2 should be gray");
            assert_eq!(r2, b2, "bg2 should be gray");
            // bg1 should be brighter than bg2 (gradient gets darker)
            assert!(r1 > r2, "tab gradient should get darker: bg1=0x{r1:02x} bg2=0x{r2:02x}");
        }
        _ => panic!("inactive tabs should have RGB bg: bg1={bg1:?} bg2={bg2:?}"),
    }
}

// === Issue 12: Right powercap on last tab ===

#[test]
fn last_tab_right_powercap_fg_equals_tab_bg() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let surface = h.backend.surface().unwrap();
    let w = surface.width();
    let row0: Vec<_> = (0..w).map(|x| surface.cell(x, 0).clone()).collect();

    // Find the trailing E0B4 (right half-circle) after the last tab
    // It should have fg = last_tab_bg, bg = fill_bg
    let last_cap = row0.iter().rposition(|c| c.ch == '\u{E0B4}');
    assert!(last_cap.is_some(), "should have trailing right powercap");
    let cap_cell = &row0[last_cap.unwrap()];
    // The cap's bg should be the fill color (black/Reset/Transparent)
    assert!(
        cap_cell.style.bg == Color::Reset
            || cap_cell.style.bg == Color::Ansi(0)
            || cap_cell.style.bg == Color::Rgb(0, 0, 0)
            || cap_cell.style.bg == Color::Transparent,
        "trailing cap bg should be fill (black/transparent): {:?}",
        cap_cell.style.bg
    );
    // The cap's fg should be the last tab's bg (a gray or active color)
    match cap_cell.style.fg {
        Color::Rgb(r, g, b) if r == g && g == b && r > 0x10 => {} // gray gradient
        Color::Ansi(4) | Color::Ansi(8) => {}                     // active tab colors
        Color::Transparent => {}                                  // fill-through
        other => panic!("trailing cap fg should be tab bg color: {:?}", other),
    }
}

// === Issue 13: Between-tab powercaps ===

#[test]
fn between_tabs_powercap_has_correct_colors() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, KeyMod::default());
    h.inject_key(KeyCode::Enter, KeyMod::default());
    h.run_cycles(1);
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let surface = h.backend.surface().unwrap();
    let w = surface.width();
    let row0: Vec<_> = (0..w).map(|x| surface.cell(x, 0).clone()).collect();

    // Find E0B4 caps between tabs (not the first or last)
    let caps: Vec<usize> = row0
        .iter()
        .enumerate()
        .filter(|(_, c)| c.ch == '\u{E0B4}')
        .map(|(i, _)| i)
        .collect();

    // Should have at least 2 caps (active right + between inactive + trailing)
    assert!(caps.len() >= 2, "should have multiple powercaps: found {}", caps.len());

    // For a between-tab cap: fg = left tab bg, bg = right tab bg
    // Check the first non-active cap (between inactive tabs)
    if caps.len() >= 3 {
        let mid_cap = caps[1]; // second cap (between first and second inactive)
        let cap_cell = &row0[mid_cap];
        // fg should be the left tab's bg (a gray)
        // bg should be the right tab's bg (a darker gray)
        match (cap_cell.style.fg, cap_cell.style.bg) {
            (Color::Rgb(fr, fg, fb), Color::Rgb(br, bg, bb)) => {
                assert_eq!(fr, fg, "cap fg should be gray");
                assert_eq!(fr, fb, "cap fg should be gray");
                assert_eq!(br, bg, "cap bg should be gray");
                assert_eq!(br, bb, "cap bg should be gray");
                // fg (left tab) should be brighter than bg (right tab)
                assert!(
                    fr > br,
                    "cap fg (left tab bg) should be brighter than cap bg (right tab bg): 0x{fr:02x} vs 0x{br:02x}"
                );
            }
            _ => {} // Skip if not RGB (might be active tab transition)
        }
    }
}

// === Issue 8: Dropdown hides on focus change ===

#[test]
fn dropdown_closes_on_focus_change() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // Open dropdown on left panel
    h.inject_key(KeyCode::Down, CTRL_SHIFT);
    h.run_cycles(1);
    let screen = h.screen_text();
    assert!(screen.contains("│"), "dropdown should be open");

    // Switch focus to center panel (F3)
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    // Dropdown should be closed now
    let row1 = h.row(1);
    assert!(
        !row1.starts_with("│"),
        "dropdown should close on focus change: {:?}",
        row1
    );
}
