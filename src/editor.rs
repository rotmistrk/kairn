use std::io;
use std::process::Command;

use anyhow::{Context, Result};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

/// Launch `$EDITOR` (or `vi`) on the given file path.
/// Suspends the TUI, runs the editor, then restores the TUI.
pub fn launch_editor(path: &str) -> Result<()> {
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vi".into());

    // Suspend TUI
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;

    // Run editor
    let status = Command::new(&editor)
        .arg(path)
        .status()
        .with_context(|| format!("failed to launch editor: {editor}"))?;

    // Restore TUI
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    if !status.success() {
        anyhow::bail!("editor exited with status: {status}");
    }

    Ok(())
}
