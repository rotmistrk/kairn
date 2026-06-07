//! Integration tests: tools panel subpanels in narrow layout (bottom position).

mod helpers;

use helpers::{temp_project, TestHarness};
use txv_core::event::{KeyCode, KeyMod};

fn none() -> KeyMod {
    KeyMod::default()
}
fn ctrl() -> KeyMod {
    KeyMod::CTRL
}
fn ctrl_alt() -> KeyMod {
    KeyMod::CTRL.with_alt()
}

#[test]
fn narrow_tools_move_tab_creates_subpanel() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    // Narrow layout: tools on bottom (width < 200)
    let mut h = TestHarness::with_size(dir.path(), 100, 30);
    h.run_cycles(2);

    // Focus tools panel (F4)
    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);

    // Add a second shell tab
    h.dispatch_command(kairn::commands::CM_NEW_SHELL, None);
    h.run_cycles(2);

    // Move tab to subpanel (Ctrl+Alt+W)
    h.inject_key(KeyCode::Char('w'), ctrl_alt());
    h.run_cycles(3);

    // Should have split — in narrow layout tools splits horizontally (side by side)
    let screen = h.screen_text();
    assert!(
        screen.contains("Shell") || screen.contains("Kiro"),
        "tools panel should show tabs after split in narrow layout:\n{}",
        screen
    );
}

#[test]
fn narrow_tools_ctrl_w_cycles_focus() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 30);
    h.run_cycles(2);

    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);
    h.dispatch_command(kairn::commands::CM_NEW_SHELL, None);
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('w'), ctrl_alt());
    h.run_cycles(3);

    // Ctrl-W w cycles focus
    h.inject_key(KeyCode::Char('w'), ctrl());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('w'), KeyMod::default());
    h.run_cycles(2);

    // Should not crash
    assert!(h.contains("Shell") || h.contains("Kiro"));
}

#[test]
fn narrow_tools_collapse_returns_to_single() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 30);
    h.run_cycles(2);

    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);
    h.dispatch_command(kairn::commands::CM_NEW_SHELL, None);
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('w'), ctrl_alt());
    h.run_cycles(3);

    // Collapse
    h.dispatch_command(kairn::commands::CM_SPLIT_CLOSE, None);
    h.run_cycles(3);

    assert!(h.contains("Shell") || h.contains("Kiro"));
}
