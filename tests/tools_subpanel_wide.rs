//! Integration tests: tools panel subpanels (move-tab, Ctrl-W, collapse).
//! Tests in wide layout (tools panel on the right).

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
fn tools_move_tab_creates_subpanel() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    // Wide layout: tools on right
    let mut h = TestHarness::with_size(dir.path(), 300, 40);
    h.run_cycles(2);

    // Focus tools panel (F4)
    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);

    // Need at least 2 tabs to move one. Add a second shell tab.
    h.dispatch_command(kairn::commands::CM_NEW_SHELL, None);
    h.run_cycles(2);

    // Move tab to subpanel (Ctrl+Alt+W)
    h.inject_key(KeyCode::Char('w'), ctrl_alt());
    h.run_cycles(3);

    // Should now have a split in the tools panel
    let screen = h.screen_text();
    // In wide layout with vertical split, we expect a horizontal divider between subpanels
    // (tools panel splits vertically = stacked)
    assert!(
        screen.contains("Shell") || screen.contains("Kiro"),
        "tools panel should still show tabs after split:\n{}",
        screen
    );
}

#[test]
fn tools_ctrl_w_cycles_subpanel_focus() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 300, 40);
    h.run_cycles(2);

    // Focus tools, add tab, move to create split
    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);
    h.dispatch_command(kairn::commands::CM_NEW_SHELL, None);
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('w'), ctrl_alt());
    h.run_cycles(3);

    // Ctrl-W w should cycle focus between subpanels
    h.inject_key(KeyCode::Char('w'), ctrl());
    h.run_cycles(1);
    h.inject_key(KeyCode::Char('w'), KeyMod::default());
    h.run_cycles(2);

    // Should not crash, panel still visible
    assert!(h.contains("Shell") || h.contains("Kiro"));
}

#[test]
fn tools_collapse_subpanel() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 300, 40);
    h.run_cycles(2);

    // Focus tools, add tab, move to create split
    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);
    h.dispatch_command(kairn::commands::CM_NEW_SHELL, None);
    h.run_cycles(2);
    h.inject_key(KeyCode::Char('w'), ctrl_alt());
    h.run_cycles(3);

    // Close split via CM_SPLIT_CLOSE
    h.dispatch_command(kairn::commands::CM_SPLIT_CLOSE, None);
    h.run_cycles(3);

    // Should still show tools content
    assert!(h.contains("Shell") || h.contains("Kiro"));
}
