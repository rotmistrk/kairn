//! PtySession — spawns a PTY process and provides poll/write/resize.

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, TryRecvError};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

/// A running PTY session with background reader thread.
pub struct PtySession {
    writer: Box<dyn Write + Send>,
    rx: Receiver<Vec<u8>>,
    master: Box<dyn MasterPty + Send>,
}

impl PtySession {
    /// Spawn a new PTY process.
    pub fn spawn(cmd: &str, args: &[&str], cwd: &Path, cols: u16, rows: u16) -> std::io::Result<Self> {
        let pair = Self::open_pty(cols, rows)?;
        Self::spawn_child(&pair, cmd, args, cwd)?;
        let writer = pair
            .master
            .take_writer()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let rx = Self::start_reader(&pair.master)?;
        Ok(Self {
            writer,
            rx,
            master: pair.master,
        })
    }

    fn open_pty(cols: u16, rows: u16) -> std::io::Result<portable_pty::PtyPair> {
        let pty_system = native_pty_system();
        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };
        pty_system
            .openpty(size)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    }

    fn spawn_child(pair: &portable_pty::PtyPair, cmd: &str, args: &[&str], cwd: &Path) -> std::io::Result<()> {
        let mut cmd_builder = CommandBuilder::new(cmd);
        cmd_builder.args(args);
        cmd_builder.cwd(cwd);
        pair.slave
            .spawn_command(cmd_builder)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        Ok(())
    }

    fn start_reader(master: &Box<dyn MasterPty + Send>) -> std::io::Result<Receiver<Vec<u8>>> {
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(rx)
    }

    /// Poll for available output. Returns combined bytes or None.
    pub fn poll(&self) -> Option<Vec<u8>> {
        let mut data = Vec::new();
        loop {
            match self.rx.try_recv() {
                Ok(chunk) => data.extend(chunk),
                Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
            }
        }
        if data.is_empty() {
            None
        } else {
            Some(data)
        }
    }

    /// Write bytes to the PTY.
    pub fn write(&mut self, data: &[u8]) {
        let _ = self.writer.write_all(data);
        let _ = self.writer.flush();
    }

    /// Resize the PTY.
    pub fn resize(&self, cols: u16, rows: u16) {
        let _ = self.master.resize(PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn spawn_echo_hello() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| "/tmp".into());
        let session = PtySession::spawn("echo", &["hello"], &cwd, 80, 24).expect("spawn failed");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut output = Vec::new();
        while Instant::now() < deadline {
            if let Some(data) = session.poll() {
                output.extend(data);
                if String::from_utf8_lossy(&output).contains("hello") {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let text = String::from_utf8_lossy(&output);
        assert!(text.contains("hello"), "expected 'hello' in output: {:?}", text);
    }

    #[test]
    fn resize_does_not_panic() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| "/tmp".into());
        let session = PtySession::spawn("cat", &[], &cwd, 80, 24).expect("spawn failed");
        session.resize(120, 40);
        session.resize(40, 10);
    }
}
