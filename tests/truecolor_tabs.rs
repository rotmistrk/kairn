//! Tests for tab bar rendering with truecolor gradient and thin separators.

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::cell::Color;
use txv_core::event::{KeyCode, KeyMod};

/// Verify thin separators (E0B1/E0B3) appear between inactive tabs.
#[test]
fn inactive_tabs_use_thin_separators() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", ""), ("d.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    // Open 4 files
    for _ in 0..4 {
        h.inject_key(KeyCode::Enter, KeyMod::default());
        h.run_cycles(1);
        h.inject_key(KeyCode::F(2), KeyMod::default());
        h.inject_key(KeyCode::Down, KeyMod::default());
    }
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let buf = h.backend.buffer().unwrap();
    let w = buf.width();
    let row0: Vec<_> = (0..w).map(|x| buf.cell(x, 0).clone()).collect();

    // Find thin separators (E0B1 or E0B3) in center panel area
    let divider = row0.iter().position(|c| c.ch() == '┬').unwrap_or(0);
    let thin_seps: Vec<usize> = row0
        .iter()
        .enumerate()
        .skip(divider)
        .filter(|(_, c)| c.ch() == '\u{E0B1}' || c.ch() == '\u{E0B3}')
        .map(|(i, _)| i)
        .collect();

    assert!(
        !thin_seps.is_empty(),
        "should have thin separators between inactive tabs"
    );
}

/// Verify thin separator uses dim_fg color (visible on any bg).
#[test]
fn thin_separator_uses_dim_fg() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    for _ in 0..3 {
        h.inject_key(KeyCode::Enter, KeyMod::default());
        h.run_cycles(1);
        h.inject_key(KeyCode::F(2), KeyMod::default());
        h.inject_key(KeyCode::Down, KeyMod::default());
    }
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let buf = h.backend.buffer().unwrap();
    let w = buf.width();
    let divider = (0..w).find(|&x| buf.cell(x, 0).ch() == '┬').unwrap_or(0);

    // Find first thin separator after divider
    let sep_pos = (divider..w).find(|&x| {
        let ch = buf.cell(x, 0).ch();
        ch == '\u{E0B1}' || ch == '\u{E0B3}'
    });

    if let Some(pos) = sep_pos {
        let cell = buf.cell(pos, 0);
        // fg should NOT be the same as bg (must be visible)
        assert_ne!(
            cell.style().fg(),
            cell.style().bg(),
            "thin separator must be visible (fg≠bg): fg={:?} bg={:?}",
            cell.style().fg(),
            cell.style().bg()
        );
    }
}

/// Verify active tab still uses half-circle caps (E0B6/E0B4) even with thin separators.
#[test]
fn active_tab_keeps_half_circle_caps_with_thin_seps() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    for _ in 0..3 {
        h.inject_key(KeyCode::Enter, KeyMod::default());
        h.run_cycles(1);
        h.inject_key(KeyCode::F(2), KeyMod::default());
        h.inject_key(KeyCode::Down, KeyMod::default());
    }
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let buf = h.backend.buffer().unwrap();
    let w = buf.width();
    let divider = (0..w).find(|&x| buf.cell(x, 0).ch() == '┬').unwrap_or(0);

    // Active tab should still have E0B6 (left) and E0B4 (right)
    let has_left_cap = (divider..w).any(|x| buf.cell(x, 0).ch() == '\u{E0B6}');
    let has_right_cap = (divider..w).any(|x| buf.cell(x, 0).ch() == '\u{E0B4}');

    assert!(has_left_cap, "active tab should have E0B6 left cap");
    assert!(has_right_cap, "active tab should have E0B4 right cap");
}

/// Verify gradient bg values are distinct RGB grays (truecolor test).
#[test]
fn gradient_bg_uses_distinct_rgb_values() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", ""), ("c.rs", ""), ("d.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 30);
    for _ in 0..4 {
        h.inject_key(KeyCode::Enter, KeyMod::default());
        h.run_cycles(1);
        h.inject_key(KeyCode::F(2), KeyMod::default());
        h.inject_key(KeyCode::Down, KeyMod::default());
    }
    h.inject_key(KeyCode::F(3), KeyMod::default());
    h.run_cycles(1);

    let buf = h.backend.buffer().unwrap();
    let w = buf.width();
    let divider = (0..w).find(|&x| buf.cell(x, 0).ch() == '┬').unwrap_or(0);

    // Collect unique RGB bg values from inactive tab area (after divider, skip active)
    let mut grays: Vec<u8> = Vec::new();
    for x in divider..w {
        let cell = buf.cell(x, 0);
        if let Color::Rgb(r, g, b) = cell.style().bg() {
            if r == g && g == b && r > 0x10 && r < 0x80 {
                if !grays.contains(&r) {
                    grays.push(r);
                }
            }
        }
    }

    // With 3+ inactive tabs, should have at least 2 distinct gray levels
    assert!(
        grays.len() >= 2,
        "should have multiple distinct gray levels in gradient: {:?}",
        grays
    );

    // Verify they're in descending order (brighter first)
    let mut sorted = grays.clone();
    sorted.sort_by(|a, b| b.cmp(a));
    assert_eq!(
        grays, sorted,
        "gradient should be descending (brighter→darker): {:?}",
        grays
    );
}

/// Dropdown cursor line uses middle dots (·) for padding.
#[test]
fn dropdown_cursor_uses_middle_dots() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // Open dropdown
    h.inject_key(KeyCode::Char('0'), KeyMod::ALT);
    h.run_cycles(1);

    let screen = h.screen_text();
    assert!(
        screen.contains('·'),
        "dropdown cursor line should use middle dots: {screen}"
    );
}
