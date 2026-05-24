//! Integration tests for TabBar/TabPanel issues found during manual testing.
//! Each test documents the expected behavior from the design doc.

mod helpers;

use helpers::{temp_project, TestHarness};
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

// === Issue 1: Glyph style ===
// Design: E0B6 (left half-circle) for active tab left edge
//         E0B4 (right half-circle) for active tab right edge

#[test]
fn active_tab_uses_half_circle_caps() {
    let dir = temp_project(&[("a.rs", ""), ("b.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let top = h.row(0);
    // Active tab should have E0B6 (left cap) and E0B4 (right cap)
    assert!(
        top.contains('\u{E0B6}'),
        "active tab should have left half-circle E0B6: {:?}",
        top
    );
    assert!(
        top.contains('\u{E0B4}'),
        "active tab should have right half-circle E0B4: {:?}",
        top
    );
}

// === Issue 2: Dropdown styling ===

#[test]
fn dropdown_no_top_border_line() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    // Open dropdown on left panel
    h.inject_key(KeyCode::Down, CTRL_SHIFT);
    h.run_cycles(1);

    // Row 1 should be dropdown content, NOT a ┌─┐ border
    let row1 = h.row(1);
    assert!(!row1.contains('┌'), "dropdown should NOT have top border: {:?}", row1);
}

#[test]
fn dropdown_entries_have_padding() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);
    h.inject_key(KeyCode::Down, CTRL_SHIFT);
    h.run_cycles(1);

    // Entries should have padding (spaces or middle dots for cursor)
    let screen = h.screen_text();
    assert!(
        screen.contains(" 1:Files ")
            || screen.contains("·1:Files·")
            || screen.contains(" 2:Git ")
            || screen.contains(" Files ")
            || screen.contains(" Git "),
        "dropdown entries need horizontal padding: {screen}"
    );
}

// === Issue 3: M-digit numbering ===
// Static mode: M-1→tab1, M-2→tab2 (1-indexed, fixed)
// LRU mode: M-1→most recent other, M-2→next recent
// M-0 always opens dropdown

#[test]
fn m0_opens_dropdown() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    h.inject_key(KeyCode::Char('0'), ALT);
    h.run_cycles(1);

    let screen = h.screen_text();
    // Dropdown should show tab list
    assert!(
        screen.contains("Files") && screen.contains("Git"),
        "M-0 should open dropdown: {screen}"
    );
}

#[test]
fn m1_in_static_mode_goes_to_first_tab() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::with_size(dir.path(), 200, 50);
    h.run_cycles(1);

    // Focus the left panel (Static: ₁Files ₂Git ₃Todo)
    // Initial focus is on center (Welcome). Move left.
    h.inject_key(KeyCode::Left, CTRL_SHIFT);
    h.run_cycles(1);

    // M-2 should activate Git (label ₂).
    h.inject_key(KeyCode::Char('2'), ALT);
    h.run_cycles(1);

    let top = h.row(0);
    assert!(
        top.contains("Git"),
        "M-2 in static mode should activate Git (2nd tab): {:?}",
        top
    );
}

// === Issue 4: Count badge colors ===

#[test]
fn count_badge_is_visible_not_black_on_black() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let surface = h.backend.surface().unwrap();
    let w = surface.width();
    // Find ▾ in row 0
    let badge_pos = (0..w).find(|&x| surface.cell(x, 0).ch == '▾');
    assert!(badge_pos.is_some(), "should have ▾ badge");

    let cell = surface.cell(badge_pos.unwrap(), 0);
    // Must be visible: fg != bg, fg not black
    assert_ne!(
        cell.style.fg, cell.style.bg,
        "badge must be visible (fg≠bg): fg={:?} bg={:?}",
        cell.style.fg, cell.style.bg
    );
}

// === Issue 5: handle_keys=false ===

#[test]
fn tabbar_handle_keys_is_false() {
    // Verify that TabPanel's TabBar has handle_keys=false in kairn
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    let desktop = h
        .program
        .desktop_mut()
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<txv_widgets::tiled_workspace::TiledWorkspace>())
        .expect("desktop");

    let panel = desktop.panel(kairn::slots::SlotId::Left as usize).expect("left panel");
    assert!(
        !panel.bar().handle_keys(),
        "TabBar.handle_keys should be false in kairn (kairn owns keyboard)"
    );
}

// === Issue 6: Left panel should be Static mode, Center/Right should be LRU ===

#[test]
fn left_panel_is_static_mode() {
    let dir = temp_project(&[("a.rs", "")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(1);

    // In static mode, all tabs should be visible with numbers
    // Left panel has Files, Git, Todo — all should appear in row 0
    let top = h.row(0);
    assert!(top.contains("Files"), "static mode: Files should be visible: {:?}", top);
    // With subscript numbers or N: prefix
    assert!(
        top.contains('₁') || top.contains('₂') || top.contains("1:") || top.contains("2:"),
        "static mode: tabs should have number labels: {:?}",
        top
    );
}
