// Command runner: execute commands, capture stdout/stderr.

use std::process::Command;

use anyhow::{Context, Result};

/// Result of running a command.
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
    pub code: Option<i32>,
}

/// Run a shell command and capture output.
/// Uses $SHELL -c "command" for shell expansion.
pub fn run_command(cmd: &str) -> Result<CommandResult> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
    let output = Command::new(&shell)
        .arg("-c")
        .arg(cmd)
        .output()
        .with_context(|| format!("running: {cmd}"))?;

    Ok(CommandResult {
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        success: output.status.success(),
        code: output.status.code(),
    })
}
