//! Tests: hardware cursor must be present after editor split.

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
    // F3 focuses editor panel
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(2);
}

/// Enter insert mode (press 'i' in normal mode).
fn enter_insert_mode(h: &mut TestHarness) {
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(1);
}

#[test]
fn cursor_present_before_split() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");
    enter_insert_mode(&mut h);

    let cursor = h.backend.cursor();
    assert!(cursor.is_some(), "cursor must be present in insert mode before split");
}

#[test]
fn cursor_present_after_split() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");
    enter_insert_mode(&mut h);

    // Split
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::horizontal())),
    );
    h.run_cycles(3);

    let cursor = h.backend.cursor();
    assert!(cursor.is_some(), "cursor must be present after split");
}

#[test]
fn cursor_present_after_vsplit() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "main.rs");
    enter_insert_mode(&mut h);

    // Vsplit
    h.dispatch_command(
        kairn::commands::CM_SPLIT,
        Some(Box::new(kairn::commands::SplitRequest::vertical())),
    );
    h.run_cycles(3);

    let cursor = h.backend.cursor();
    assert!(cursor.is_some(), "cursor must be present after vsplit");
}

#[test]
fn focus_tools_in_narrow_layout() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    // Width < 200 triggers narrow layout
    let mut h = TestHarness::with_size(dir.path(), 100, 30);
    h.run_cycles(2);

    // Focus should start on center (editor) panel
    // Try Ctrl+Shift+Down to reach tools at bottom
    h.inject_key(
        KeyCode::Down,
        KeyMod {
            ctrl: true,
            shift: true,
            alt: false,
        },
    );
    h.run_cycles(2);

    // Tools panel should now be focused — Shell tab should be active
    let screen = h.screen_text();
    // The status bar shows the focused panel info
    assert!(
        screen.contains("Shell") || screen.contains("Kiro"),
        "tools panel should be reachable in narrow layout:\n{}",
        screen
    );
}

#[test]
fn tools_subpanel_divider_in_narrow_layout() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    // Narrow layout (width < 200)
    let mut h = TestHarness::with_size(dir.path(), 100, 30);
    h.run_cycles(2);

    // Focus tools panel (F4)
    h.inject_key(KeyCode::F(4), none());
    h.run_cycles(2);

    // Add a second tab to tools panel so we can split
    {
        let desktop = h.program.desktop_mut();
        if let Some(any) = desktop.as_any_mut() {
            if let Some(ws) = any.downcast_mut::<txv_widgets::tiled_workspace::TiledWorkspace>() {
                ws.insert_tab(2, "Tab2", Box::new(txv_widgets::text_area::TextArea::new()));
            }
        }
    }
    h.run_cycles(1);

    // Move tab to create subpanel split (Ctrl+Alt+W)
    h.inject_key(
        KeyCode::Char('w'),
        KeyMod {
            ctrl: true,
            alt: true,
            shift: false,
        },
    );
    h.run_cycles(2);

    let screen = h.screen_text();
    // In narrow layout, tools is horizontal split — should have vertical divider
    let chrome_line = screen
        .lines()
        .position(|l| l.contains("Shell") || l.contains("Tab2"))
        .unwrap_or(0);
    let tools_area: String = screen.lines().skip(chrome_line + 1).collect::<Vec<_>>().join("\n");
    assert!(
        tools_area.contains("│"),
        "vertical divider missing between tools subpanels in narrow layout:\n{}",
        screen
    );
}

#[test]
fn shrink_main_panel_narrow_layout() {
    let dir = temp_project(&[("main.rs", "fn main() {}\n")]);
    let mut h = TestHarness::with_size(dir.path(), 100, 30);
    h.run_cycles(2);

    // Focus main panel (F3)
    h.inject_key(KeyCode::F(3), none());
    h.run_cycles(1);

    // Get initial tools panel height
    let screen1 = h.screen_text();
    let chrome_line1 = screen1.lines().position(|l| l.contains("Shell")).unwrap_or(0);

    // Shrink main vertically (M-S-Up = CM_TW_SHRINK_V moves boundary up)
    use txv_widgets::tiled_workspace::commands::CM_TW_SHRINK_V;
    h.program.sink().push_command(CM_TW_SHRINK_V, None);
    h.run_cycles(1);

    let screen2 = h.screen_text();
    let chrome_line2 = screen2.lines().position(|l| l.contains("Shell")).unwrap_or(0);

    assert!(
        chrome_line2 < chrome_line1,
        "shrink should move tools chrome up: before={}, after={}",
        chrome_line1,
        chrome_line2
    );
}
