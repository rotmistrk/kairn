//! Tests for mode-dependent cursor shapes.

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

/// Check if there's a reverse-style cell in the editor content area (line 1 of file).
/// This indicates the software cursor is being drawn.
fn has_software_cursor(h: &TestHarness) -> bool {
    let surface = h.backend.surface().expect("no surface");
    let w = surface.width();
    let height = surface.height();
    // Find the row with line number "1" — that's where the cursor should be
    for y in 1..height.saturating_sub(1) {
        // Look for the '│' divider followed by line number
        for x in 0..w.saturating_sub(3) {
            let c = surface.cell(x, y);
            if c.ch == '│' {
                // Check if next non-space char is '1'
                let mut nx = x + 1;
                while nx < w && surface.cell(nx, y).ch == ' ' {
                    nx += 1;
                }
                if nx < w && surface.cell(nx, y).ch == '1' {
                    // Found line 1 row. Now check for reverse cells after the gutter
                    let content_start = nx + 2; // past "1 "
                    for cx in content_start..w {
                        if surface.cell(cx, y).style.bg == txv_core::cell::Color::Ansi(7)
                            && surface.cell(cx, y).style.fg == txv_core::cell::Color::Ansi(0)
                        {
                            return true;
                        }
                    }
                    return false;
                }
            }
        }
    }
    false
}

/// Normal mode uses software cursor (visible as reverse-video cell).
#[test]
fn normal_mode_has_software_cursor() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");
    h.run_cycles(2);

    assert!(
        has_software_cursor(&h),
        "software cursor should be visible in normal mode"
    );
}

/// Insert mode uses hardware cursor (no software cursor drawn).
#[test]
fn insert_mode_hides_software_cursor() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Enter insert mode
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(2);

    assert!(
        !has_software_cursor(&h),
        "software cursor should NOT be visible in insert mode"
    );
}

/// :set cursor_normal=bar switches normal mode to hardware cursor.
#[test]
fn set_cursor_normal_to_hardware() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Set normal mode to use hardware bar cursor
    h.inject_key(KeyCode::Char(':'), none());
    h.run_cycles(1);
    h.inject_str("set cursor_normal=bar");
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);

    assert!(
        !has_software_cursor(&h),
        "software cursor should NOT be visible after cursor_normal=bar"
    );
}

/// :set cursor_insert=software switches insert mode to software cursor.
#[test]
fn set_cursor_insert_to_software() {
    let dir = temp_project(&[("f.rs", "hello\n")]);
    let mut h = TestHarness::new(dir.path());
    h.run_cycles(2);
    open_and_focus(&mut h, dir.path(), "f.rs");

    // Set insert mode to use software cursor
    h.inject_key(KeyCode::Char(':'), none());
    h.run_cycles(1);
    h.inject_str("set cursor_insert=software");
    h.inject_key(KeyCode::Enter, none());
    h.run_cycles(2);

    // Enter insert mode
    h.inject_key(KeyCode::Char('i'), none());
    h.run_cycles(2);

    assert!(
        has_software_cursor(&h),
        "software cursor should be visible with cursor_insert=software"
    );
}
