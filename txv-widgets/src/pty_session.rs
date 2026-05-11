//! PtySession — spawns a PTY process and provides poll/write/resize.

use std::io::{Read, Write};
use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::thread;

use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};

/// A running PTY session with background reader and writer threads.
pub struct PtySession {
    write_tx: mpsc::Sender<Vec<u8>>,
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
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        let rx = Self::start_reader(&pair.master)?;
        let write_tx = Self::start_writer(writer);
        Ok(Self {
            write_tx,
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
            .map_err(|e| std::io::Error::other(e.to_string()))
    }

    fn spawn_child(pair: &portable_pty::PtyPair, cmd: &str, args: &[&str], cwd: &Path) -> std::io::Result<()> {
        let mut cmd_builder = CommandBuilder::new(cmd);
        cmd_builder.args(args);
        cmd_builder.cwd(cwd);
        cmd_builder.env("TERM", "xterm-256color");
        pair.slave
            .spawn_command(cmd_builder)
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        Ok(())
    }

    #[allow(clippy::borrowed_box)]
    fn start_reader(master: &Box<dyn MasterPty + Send>) -> std::io::Result<Receiver<Vec<u8>>> {
        let mut reader = master
            .try_clone_reader()
            .map_err(|e| std::io::Error::other(e.to_string()))?;
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
    /// After this returns None, call is_alive() to check if process exited.
    pub fn poll(&self) -> Option<Vec<u8>> {
        let mut data = Vec::new();
        while let Ok(chunk) = self.rx.try_recv() {
            data.extend(chunk);
        }
        if data.is_empty() {
            None
        } else {
            Some(data)
        }
    }

    /// Check if the PTY process is still running.
    /// Only meaningful after poll() returns None (no pending data).
    pub fn is_alive(&self) -> bool {
        use std::sync::mpsc::TryRecvError;
        // If poll() already drained everything, try_recv will tell us channel state
        match self.rx.try_recv() {
            Err(TryRecvError::Disconnected) => false,
            _ => true, // Empty or Ok (shouldn't happen after poll drained)
        }
    }

    /// Write bytes to the PTY (non-blocking — queued to background thread).
    pub fn write(&self, data: &[u8]) {
        let _ = self.write_tx.send(data.to_vec());
    }

    fn start_writer(mut writer: Box<dyn Write + Send>) -> mpsc::Sender<Vec<u8>> {
        let (tx, rx) = mpsc::channel::<Vec<u8>>();
        thread::spawn(move || {
            while let Ok(data) = rx.recv() {
                if writer.write_all(&data).is_err() {
                    break;
                }
                let _ = writer.flush();
            }
        });
        tx
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

    #[test]
    fn term_is_xterm_256color() {
        let cwd = std::env::current_dir().unwrap_or_else(|_| "/tmp".into());
        let session = PtySession::spawn("sh", &["-c", "echo TERM=$TERM"], &cwd, 80, 24).expect("spawn failed");
        let deadline = Instant::now() + Duration::from_secs(3);
        let mut output = Vec::new();
        while Instant::now() < deadline {
            if let Some(data) = session.poll() {
                output.extend(data);
                if String::from_utf8_lossy(&output).contains("TERM=") {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let text = String::from_utf8_lossy(&output);
        assert!(
            text.contains("TERM=xterm-256color"),
            "expected TERM=xterm-256color in output: {:?}",
            text
        );
    }
}
