//! PtyTerminal — a View that owns a TermBuf + PtySession.

use std::path::Path;

use txv_core::event::Event;
use txv_core::prelude::*;
use txv_render::termbuf::TermBuf;

use crate::key_encode::key_to_bytes;
use crate::pty_session::PtySession;

/// Terminal view backed by a real PTY process.
pub struct PtyTerminal {
    state: ViewState,
    termbuf: TermBuf,
    session: Option<PtySession>,
    title: String,
    prev_cols: u16,
    prev_rows: u16,
}

impl PtyTerminal {
    /// Spawn the user's default shell.
    pub fn spawn_shell(cols: u16, rows: u16) -> std::io::Result<Self> {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into());
        let cwd = std::env::current_dir().unwrap_or_else(|_| "/".into());
        let session = PtySession::spawn(&shell, &[], &cwd, cols, rows)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::new(cols, rows),
            session: Some(session),
            title: "Shell".into(),
            prev_cols: cols,
            prev_rows: rows,
        })
    }

    /// Spawn a specific command.
    pub fn spawn_command(cmd: &str, args: &[&str], cwd: &Path, cols: u16, rows: u16) -> std::io::Result<Self> {
        let session = PtySession::spawn(cmd, args, cwd, cols, rows)?;
        Ok(Self {
            state: ViewState::default(),
            termbuf: TermBuf::new(cols, rows),
            session: Some(session),
            title: cmd.into(),
            prev_cols: cols,
            prev_rows: rows,
        })
    }

    fn poll_and_feed(&mut self) {
        let Some(session) = self.session.as_mut() else {
            return;
        };
        if let Some(data) = session.poll() {
            self.termbuf.process(&data);
            self.state.dirty = true;
        }
        for resp in self.termbuf.drain_responses() {
            session.write(&resp);
        }
    }
}

impl View for PtyTerminal {
    delegate_view_state!(state, override { title, set_bounds, needs_redraw });

    fn title(&self) -> &str {
        &self.title
    }

    fn needs_redraw(&self) -> bool {
        self.state.dirty || self.session.is_some()
    }

    fn set_bounds(&mut self, r: Rect) {
        self.state.bounds = r;
        self.state.dirty = true;
        let cols = r.w;
        let rows = r.h;
        if cols > 0 && rows > 0 && (cols != self.prev_cols || rows != self.prev_rows) {
            self.prev_cols = cols;
            self.prev_rows = rows;
            self.termbuf.resize(cols, rows);
            if let Some(session) = &self.session {
                session.resize(cols, rows);
            }
        }
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds;
        if b.w == 0 || b.h == 0 {
            return;
        }
        self.termbuf.render_at(surface, b.x, b.y, b.w, b.h);
        if self.termbuf.cursor_visible() {
            let (cx, cy) = self.termbuf.cursor();
            let sx = b.x + cx;
            let sy = b.y + cy;
            if sx < surface.width() && sy < surface.height() {
                let cell = surface.cell(sx, sy);
                let mut style = cell.style;
                style.attrs.reverse = !style.attrs.reverse;
                let ch = cell.ch;
                surface.put(sx, sy, ch, style);
            }
        }
    }

    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        match event {
            Event::Tick => {
                self.poll_and_feed();
                HandleResult::Ignored
            }
            Event::Key(key) => {
                if let Some(bytes) = key_to_bytes(key) {
                    if let Some(session) = self.session.as_mut() {
                        session.write(&bytes);
                    }
                    HandleResult::Consumed
                } else {
                    HandleResult::Ignored
                }
            }
            _ => HandleResult::Ignored,
        }
    }
}
