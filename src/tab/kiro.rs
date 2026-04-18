// Kiro subprocess backend: spawn kiro-cli chat, pipe stdin/stdout.

use std::io::{BufReader, Read, Write};
use std::process::{Child, Command, Stdio};

use anyhow::{Context, Result};

/// A live kiro-cli subprocess.
pub struct KiroProcess {
    child: Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl KiroProcess {
    /// Spawn `kiro-cli chat` (or custom command).
    pub fn spawn(kiro_cmd: &str) -> Result<Self> {
        let mut child = Command::new(kiro_cmd)
            .arg("chat")
            .env("TERM", "dumb")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("spawning {kiro_cmd}"))?;

        let stdin = child.stdin.take().context("taking kiro stdin")?;
        let stdout = child.stdout.take().context("taking kiro stdout")?;

        Ok(Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
        })
    }

    /// Send a line of input to kiro.
    pub fn send_line(&mut self, line: &str) -> Result<()> {
        writeln!(self.stdin, "{line}")?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Non-blocking read of available output.
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let inner = self.reader.get_mut();
        match inner.read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if the process is still running.
    pub fn is_alive(&mut self) -> bool {
        matches!(self.child.try_wait(), Ok(None))
    }
}

impl Drop for KiroProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}
