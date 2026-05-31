//! Suspend to shell and peek screen.

use std::env;
use std::process::Command;

use crossterm::event;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};

pub(crate) fn suspend_to_shell() -> Result<(), String> {
    // Nesting guard: if KAIRN_SUSPENDED is set, we're already inside a suspend
    if env::var("KAIRN_SUSPENDED").is_ok() {
        return Err("already inside a suspended session".into());
    }
    // Leave TUI
    let _ = disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::event::DisableBracketedPaste,
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen
    );
    // Spawn shell with guard env var
    let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let result = Command::new(&shell).env("KAIRN_SUSPENDED", "1").status();
    // Re-enter TUI
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide,
        crossterm::event::EnableBracketedPaste
    );
    let _ = enable_raw_mode();
    result.map(|_| ()).map_err(|e| format!("spawn {shell}: {e}"))
}

pub(crate) fn peek_screen() {
    // Temporarily leave alternate screen to show normal terminal
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    );
    // Wait for any key
    let _ = event::read();
    // Re-enter
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    );
}
