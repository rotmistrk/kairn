// KAIRN_CAPTURE named pipe: allows shell commands to send output to main panel.
// Usage from shell: some_command > $KAIRN_CAPTURE

use std::fs;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// Manages the KAIRN_CAPTURE named pipe.
pub struct CapturePipe {
    path: PathBuf,
    reader: Option<std::fs::File>,
}

impl CapturePipe {
    /// Create the named pipe and set KAIRN_CAPTURE env var.
    pub fn create(workspace: &Path) -> Result<Self> {
        let path = workspace.join(".kairn.capture");

        // Remove stale pipe if it exists
        let _ = fs::remove_file(&path);

        // Create FIFO
        nix::unistd::mkfifo(&path, nix::sys::stat::Mode::S_IRWXU)
            .with_context(|| format!("creating FIFO: {}", path.display()))?;

        std::env::set_var("KAIRN_CAPTURE", &path);

        Ok(Self { path, reader: None })
    }

    /// Non-blocking read from the pipe. Returns captured text if any.
    pub fn poll(&mut self) -> Option<String> {
        // Open pipe non-blocking on first poll (and reopen after each EOF)
        if self.reader.is_none() {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .custom_flags(libc::O_NONBLOCK)
                .open(&self.path)
                .ok()?;
            self.reader = Some(file);
        }

        let reader = self.reader.as_mut()?;
        let mut buf = [0u8; 8192];
        match reader.read(&mut buf) {
            Ok(0) => {
                // EOF — writer closed. Reopen for next write.
                self.reader = None;
                None
            }
            Ok(n) => {
                let text = String::from_utf8_lossy(&buf[..n]).to_string();
                Some(text)
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => None,
            Err(_) => {
                self.reader = None;
                None
            }
        }
    }

    /// Clean up the pipe.
    pub fn cleanup(&self) {
        let _ = fs::remove_file(&self.path);
    }
}

impl Drop for CapturePipe {
    fn drop(&mut self) {
        self.cleanup();
    }
}
