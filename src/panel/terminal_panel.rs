//! Terminal panel: wraps txv TermBuf + portable-pty for PTY terminals.

use std::io::Write;
use std::sync::mpsc;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use txv::surface::Surface;
use txv::termbuf::TermBuf;
use txv_widgets::{EventResult, Pollable, Widget};

/// Channel-based PTY reader that implements Pollable.
pub struct PtyPoller {
    rx: mpsc::Receiver<Vec<u8>>,
}

impl Pollable for PtyPoller {
    fn poll(&mut self) -> Option<Vec<u8>> {
        // Drain all available data without blocking.
        let mut combined = Vec::new();
        while let Ok(data) = self.rx.try_recv() {
            combined.extend(data);
        }
        if combined.is_empty() {
            None
        } else {
            Some(combined)
        }
    }
}

/// A PTY-backed terminal panel.
pub struct TerminalPanel {
    term_buf: TermBuf,
    pty_writer: Option<Box<dyn Write + Send>>,
    title: String,
    cols: u16,
    rows: u16,
}

impl TerminalPanel {
    /// Create a new terminal panel with given dimensions.
    pub fn new(title: &str, cols: u16, rows: u16) -> Self {
        Self {
            term_buf: TermBuf::new(cols, rows),
            pty_writer: None,
            title: title.to_string(),
            cols,
            rows,
        }
    }

    /// Spawn a shell process. Returns a Pollable for the EventLoop.
    pub fn spawn_shell(&mut self) -> anyhow::Result<PtyPoller> {
        self.spawn_command(None)
    }

    /// Spawn a kiro-cli process. Returns a Pollable for the EventLoop.
    pub fn spawn_kiro(&mut self, kiro_cmd: &str) -> anyhow::Result<PtyPoller> {
        self.spawn_command(Some(kiro_cmd))
    }

    /// Spawn a command (shell if None, specific command if Some).
    fn spawn_command(&mut self, cmd: Option<&str>) -> anyhow::Result<PtyPoller> {
        use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

        let pty_system = NativePtySystem::default();
        let pair = pty_system.openpty(PtySize {
            rows: self.rows,
            cols: self.cols,
            pixel_width: 0,
            pixel_height: 0,
        })?;

        let mut command = match cmd {
            Some(c) => {
                let parts: Vec<&str> = c.split_whitespace().collect();
                if parts.is_empty() {
                    anyhow::bail!("empty command");
                }
                let mut cb = CommandBuilder::new(parts[0]);
                for arg in &parts[1..] {
                    cb.arg(arg);
                }
                cb
            }
            None => {
                let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
                CommandBuilder::new(shell)
            }
        };

        command.env("TERM", "xterm-256color");

        let _child = pair.slave.spawn_command(command)?;
        drop(pair.slave);

        let writer = pair.master.take_writer()?;
        self.pty_writer = Some(writer);

        // Spawn background thread to read from PTY.
        let mut reader = pair.master.try_clone_reader()?;
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(PtyPoller { rx })
    }

    /// Process raw bytes from the PTY.
    pub fn process_output(&mut self, data: &[u8]) {
        self.term_buf.process(data);
    }

    /// Send raw bytes to the PTY.
    pub fn send_input(&mut self, data: &[u8]) {
        if let Some(ref mut writer) = self.pty_writer {
            let _ = writer.write_all(data);
            let _ = writer.flush();
        }
    }

    /// Resize the terminal.
    pub fn resize(&mut self, cols: u16, rows: u16) {
        self.cols = cols;
        self.rows = rows;
        self.term_buf.resize(cols, rows);
    }

    /// Get the title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Drain PTY responses (DA1, CPR, etc.).
    pub fn drain_responses(&mut self) -> Vec<Vec<u8>> {
        self.term_buf.drain_responses()
    }

    /// Translate a key event to bytes for the PTY.
    fn key_to_bytes(key: &KeyEvent) -> Option<Vec<u8>> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char(c) if ctrl => {
                let byte = (c as u8).wrapping_sub(b'a').wrapping_add(1);
                Some(vec![byte])
            }
            KeyCode::Char(c) => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                Some(s.as_bytes().to_vec())
            }
            KeyCode::Enter => Some(vec![b'\r']),
            KeyCode::Backspace => Some(vec![0x7f]),
            KeyCode::Tab => Some(vec![b'\t']),
            KeyCode::Esc => Some(vec![0x1b]),
            KeyCode::Up => Some(b"\x1b[A".to_vec()),
            KeyCode::Down => Some(b"\x1b[B".to_vec()),
            KeyCode::Right => Some(b"\x1b[C".to_vec()),
            KeyCode::Left => Some(b"\x1b[D".to_vec()),
            KeyCode::Home => Some(b"\x1b[H".to_vec()),
            KeyCode::End => Some(b"\x1b[F".to_vec()),
            KeyCode::PageUp => Some(b"\x1b[5~".to_vec()),
            KeyCode::PageDown => Some(b"\x1b[6~".to_vec()),
            KeyCode::Delete => Some(b"\x1b[3~".to_vec()),
            KeyCode::Insert => Some(b"\x1b[2~".to_vec()),
            KeyCode::F(n) => f_key_seq(n),
            _ => None,
        }
    }
}

/// Map F-key number to escape sequence.
fn f_key_seq(n: u8) -> Option<Vec<u8>> {
    let seq = match n {
        1 => "\x1bOP",
        2 => "\x1bOQ",
        3 => "\x1bOR",
        4 => "\x1bOS",
        5 => "\x1b[15~",
        6 => "\x1b[17~",
        7 => "\x1b[18~",
        8 => "\x1b[19~",
        9 => "\x1b[20~",
        10 => "\x1b[21~",
        11 => "\x1b[23~",
        12 => "\x1b[24~",
        _ => return None,
    };
    Some(seq.as_bytes().to_vec())
}

impl Widget for TerminalPanel {
    fn render(&self, surface: &mut Surface<'_>, _focused: bool) {
        self.term_buf.render_to(surface);
    }

    fn handle_key(&mut self, key: KeyEvent) -> EventResult {
        if let Some(bytes) = Self::key_to_bytes(&key) {
            self.send_input(&bytes);
            // Send back any responses (DA1, etc.).
            for resp in self.drain_responses() {
                self.send_input(&resp);
            }
            EventResult::Consumed
        } else {
            EventResult::Ignored
        }
    }

    fn focusable(&self) -> bool {
        true
    }
}
