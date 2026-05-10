//! Suspend to shell and peek screen.

pub(crate) fn suspend_to_shell() {
    // Nesting guard: if KAIRN_SUSPENDED is set, we're already inside a suspend
    if std::env::var("KAIRN_SUSPENDED").is_ok() {
        return;
    }
    // Leave TUI
    let _ = crossterm::terminal::disable_raw_mode();
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::event::DisableBracketedPaste,
        crossterm::cursor::Show,
        crossterm::terminal::LeaveAlternateScreen
    );
    // Spawn shell with guard env var
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let _ = std::process::Command::new(&shell).env("KAIRN_SUSPENDED", "1").status();
    // Re-enter TUI
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide,
        crossterm::event::EnableBracketedPaste
    );
    let _ = crossterm::terminal::enable_raw_mode();
}

pub(crate) fn peek_screen() {
    // Temporarily leave alternate screen to show normal terminal
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::LeaveAlternateScreen,
        crossterm::cursor::Show
    );
    // Wait for any key
    let _ = crossterm::event::read();
    // Re-enter
    let _ = crossterm::execute!(
        std::io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
        crossterm::cursor::Hide
    );
}
