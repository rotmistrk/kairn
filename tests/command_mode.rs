mod helpers;

use helpers::{run_and_capture, setup, temp_project};
use txv_core::event::{KeyCode, KeyMod};

#[test]
fn alt_x_opens_prompt() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    // Alt-x opens command prompt
    be.inject_key(
        KeyCode::Char('x'),
        KeyMod { ctrl: false, alt: true, shift: false },
    );
    run_and_capture(&mut app, &mut be, 1);
    // Status bar should show ":" prompt
    let last_row = be.row(23);
    assert!(last_row.contains(":"), "expected prompt, got: {}", last_row);
}

#[test]
fn esc_cancels_prompt() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Open prompt
    be.inject_key(
        KeyCode::Char('x'),
        KeyMod { ctrl: false, alt: true, shift: false },
    );
    run_and_capture(&mut app, &mut be, 1);
    // Cancel with Esc
    be.inject_key(KeyCode::Esc, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Should be back to normal status bar
    let last_row = be.row(23);
    assert!(last_row.contains("F1:Help"));
}

#[test]
fn quit_command_emits_quit() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Open prompt and type "quit"
    be.inject_key(
        KeyCode::Char('x'),
        KeyMod { ctrl: false, alt: true, shift: false },
    );
    be.inject_str("quit\n");
    // run_cycles handles CM_QUIT by returning early
    run_and_capture(&mut app, &mut be, 2);
    // If we get here without hanging, quit worked
}

#[test]
fn tab_completes_command() {
    let dir = temp_project(&[("a.rs", "")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    // Open prompt, type "he", press Tab
    be.inject_key(
        KeyCode::Char('x'),
        KeyMod { ctrl: false, alt: true, shift: false },
    );
    be.inject_str("he");
    be.inject_key(KeyCode::Tab, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Should have completed to "help"
    let last_row = be.row(23);
    assert!(last_row.contains("help"), "expected completion, got: {}", last_row);
}

// --- BUG 1: M-x "open README.md" does nothing ---
#[test]
fn open_command_opens_file() {
    let dir = temp_project(&[("README.md", "# Hello World\nThis is content.")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    // M-x open README.md
    be.inject_key(
        KeyCode::Char('x'),
        KeyMod { ctrl: false, alt: true, shift: false },
    );
    be.inject_str("open README.md\n");
    run_and_capture(&mut app, &mut be, 2);
    // Center slot should have a tab titled "README.md"
    // and screen should contain file content
    let screen = be.screen_text();
    assert!(
        screen.contains("README.md"),
        "expected tab title 'README.md' on screen, got:\n{}",
        screen
    );
    assert!(
        screen.contains("Hello World"),
        "expected file content on screen, got:\n{}",
        screen
    );
}

// --- BUG 3: Tab completion not showing for file paths ---
#[test]
fn tab_completes_file_path() {
    let dir = temp_project(&[("README.md", "content")]);
    let (mut app, mut be) = setup(dir.path(), 80, 24);
    run_and_capture(&mut app, &mut be, 1);
    // M-x, type "open READ", Tab
    be.inject_key(
        KeyCode::Char('x'),
        KeyMod { ctrl: false, alt: true, shift: false },
    );
    be.inject_str("open READ");
    be.inject_key(KeyCode::Tab, KeyMod::default());
    run_and_capture(&mut app, &mut be, 1);
    // Prompt should now contain "open README.md" (completed)
    let last_row = be.row(23);
    assert!(
        last_row.contains("README.md"),
        "expected completed path in prompt, got: {}",
        last_row
    );
}
