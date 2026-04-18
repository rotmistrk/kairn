// PTY shell backend using portable-pty.

use std::io::{Read, Write};

use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

/// A live PTY shell process.
pub struct PtyShell {
    master: Box<dyn MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
}

impl PtyShell {
    /// Spawn a shell in a new PTY.
    pub fn spawn(shell: &str, cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("opening PTY")?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");

        pair.slave.spawn_command(cmd).context("spawning shell")?;

        // Drop slave — master owns the connection
        drop(pair.slave);

        let reader = pair
            .master
            .try_clone_reader()
            .context("cloning PTY reader")?;
        let writer = pair.master.take_writer().context("taking PTY writer")?;

        Ok(Self {
            master: pair.master,
            reader,
            writer,
        })
    }

    /// Non-blocking read from PTY. Returns empty vec if nothing available.
    pub fn try_read(&mut self, buf: &mut [u8]) -> Result<usize> {
        // portable-pty reader is blocking by default.
        // We use a small buffer and set_non_blocking isn't available,
        // so we rely on the polling loop calling this frequently.
        // The reader will block briefly — we handle this at the
        // event loop level with poll timeout.
        match self.reader.read(buf) {
            Ok(n) => Ok(n),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => Ok(0),
            Err(e) => Err(e.into()),
        }
    }

    /// Write raw bytes to the PTY (keystrokes).
    pub fn write_all(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data)?;
        self.writer.flush()?;
        Ok(())
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) -> Result<()> {
        self.master
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .context("resizing PTY")?;
        Ok(())
    }
}
