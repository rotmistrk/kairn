//! TerminalView — PTY + TermBuf as a View.
//!
//! Owns a pseudo-terminal child process and a VTE-driven terminal buffer.
//! Forwards all key events to the PTY. Renders TermBuf to the surface.
//! Polls PTY output via Data events.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use txv::layout::Rect;
use txv::surface::Surface;
use txv::termbuf::TermBuf;
use txv_widgets::view::{DrawContext, Event, HandleResult, View};

use crate::types::CommandOutbox;

/// Shared PTY reader state for polling.
struct PtyReader {
    data: Vec<u8>,
}

/// Terminal view: PTY + VTE buffer.
pub struct TerminalView {
    termbuf: TermBuf,
    writer: Box<dyn Write + Send>,
    reader_buf: Arc<Mutex<PtyReader>>,
    title: String,
    bounds: Rect,
    pub outbox: CommandOutbox,
}

impl TerminalView {
    /// Spawn a shell in a new PTY.
    ///
    /// Returns `None` if the PTY cannot be created.
    pub fn spawn_shell(title: &str, cols: u16, rows: u16) -> Option<Self> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Self::spawn_command(title, &shell, &[], cols, rows)
    }

    /// Spawn a command in a new PTY.
    pub fn spawn_command(
        title: &str,
        program: &str,
        args: &[&str],
        cols: u16,
        rows: u16,
    ) -> Option<Self> {
        use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

        let pty_system = NativePtySystem::default();
        let pair = pty_system
            .openpty(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .ok()?;

        let mut cmd = CommandBuilder::new(program);
        for arg in args {
            cmd.arg(*arg);
        }
        cmd.env("TERM", "xterm-256color");
        if let Ok(dir) = std::env::current_dir() {
            cmd.cwd(dir);
        }

        pair.slave.spawn_command(cmd).ok()?;
        drop(pair.slave);

        let writer = pair.master.take_writer().ok()?;
        let mut reader = pair.master.try_clone_reader().ok()?;
        // Keep master alive
        let _master = pair.master;

        let reader_buf = Arc::new(Mutex::new(PtyReader { data: Vec::new() }));
        let reader_clone = Arc::clone(&reader_buf);

        // Background thread reads PTY output
        thread::spawn(move || {
            let mut buf = [0u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(mut rb) = reader_clone.lock() {
                            rb.data.extend_from_slice(&buf[..n]);
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Some(Self {
            termbuf: TermBuf::new(cols, rows),
            writer,
            reader_buf,
            title: title.to_string(),
            bounds: Rect { x: 0, y: 0, w: cols, h: rows },
            outbox: CommandOutbox::default(),
        })
    }

    /// The tab title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Poll for new PTY data and feed it to the terminal buffer.
    fn poll_pty(&mut self) {
        let data = {
            let mut rb = match self.reader_buf.lock() {
                Ok(rb) => rb,
                Err(_) => return,
            };
            if rb.data.is_empty() {
                return;
            }
            std::mem::take(&mut rb.data)
        };
        self.termbuf.process(&data);

        // Send any terminal responses back to PTY
        for response in self.termbuf.drain_responses() {
            let _ = self.writer.write_all(&response);
        }
    }

    /// Encode a key event as bytes to send to the PTY.
    fn encode_key(key: &KeyEvent) -> Vec<u8> {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Char(c) if ctrl => {
                let byte = (c as u8).wrapping_sub(b'a').wrapping_add(1);
                vec![byte]
            }
            KeyCode::Char(c) => {
                let mut buf = [0u8; 4];
                let s = c.encode_utf8(&mut buf);
                s.as_bytes().to_vec()
            }
            KeyCode::Enter => vec![b'\r'],
            KeyCode::Backspace => vec![0x7f],
            KeyCode::Tab => vec![b'\t'],
            KeyCode::Esc => vec![0x1b],
            KeyCode::Up => b"\x1b[A".to_vec(),
            KeyCode::Down => b"\x1b[B".to_vec(),
            KeyCode::Right => b"\x1b[C".to_vec(),
            KeyCode::Left => b"\x1b[D".to_vec(),
            KeyCode::Home => b"\x1b[H".to_vec(),
            KeyCode::End => b"\x1b[F".to_vec(),
            KeyCode::PageUp => b"\x1b[5~".to_vec(),
            KeyCode::PageDown => b"\x1b[6~".to_vec(),
            KeyCode::Delete => b"\x1b[3~".to_vec(),
            KeyCode::Insert => b"\x1b[2~".to_vec(),
            KeyCode::F(n) => match n {
                1 => b"\x1bOP".to_vec(),
                2 => b"\x1bOQ".to_vec(),
                3 => b"\x1bOR".to_vec(),
                4 => b"\x1bOS".to_vec(),
                5..=8 => format!("\x1b[1{}~", n + 10).into_bytes(),
                _ => format!("\x1b[{}~", n + 11).into_bytes(),
            },
            _ => Vec::new(),
        }
    }
}

impl View for TerminalView {
    fn draw(&self, surface: &mut Surface<'_>, _ctx: &DrawContext) {
        self.termbuf.render_to(surface);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        match event {
            Event::Key(key) => {
                let bytes = Self::encode_key(key);
                if !bytes.is_empty() {
                    let _ = self.writer.write_all(&bytes);
                    let _ = self.writer.flush();
                }
                HandleResult::Consumed
            }
            Event::Tick => {
                self.poll_pty();
                HandleResult::Ignored
            }
            Event::Data { payload, .. } => {
                self.termbuf.process(payload);
                HandleResult::Consumed
            }
            Event::Resize(w, h) => {
                self.termbuf.resize(*w, *h);
                HandleResult::Consumed
            }
            _ => HandleResult::Ignored,
        }
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn set_bounds(&mut self, rect: Rect) {
        self.bounds = rect;
        if rect.w > 0 && rect.h > 0 {
            self.termbuf.resize(rect.w, rect.h);
        }
    }
}
