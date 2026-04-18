// Kiro subprocess backend: spawn kiro-cli chat, pipe stdin/stdout/stderr.

use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result};

/// A live kiro-cli subprocess.
pub struct KiroProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    stdout: std::process::ChildStdout,
    stderr: std::process::ChildStderr,
}

impl KiroProcess {
    pub fn spawn(kiro_cmd: &str) -> Result<Self> {
        let mut child = Command::new(kiro_cmd)
            .arg("chat")
            .env("TERM", "dumb")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("spawning {kiro_cmd}"))?;

        let stdin = child.stdin.take().context("taking kiro stdin")?;
        let stdout = child.stdout.take().context("taking kiro stdout")?;
        let stderr = child.stderr.take().context("taking kiro stderr")?;

        Ok(Self {
            child,
            stdin,
            stdout,
            stderr,
        })
    }

    pub fn send_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()?;
        Ok(())
    }

    pub fn try_read_stdout(&mut self, buf: &mut [u8]) -> usize {
        self.stdout.read(buf).unwrap_or_default()
    }

    pub fn try_read_stderr(&mut self, buf: &mut [u8]) -> usize {
        self.stderr.read(buf).unwrap_or_default()
    }

    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for KiroProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
