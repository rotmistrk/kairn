//! MessagesView — displays application message log from shared ring buffer.

use std::sync::{Arc, Mutex};

use txv_core::prelude::*;

use crate::message_ring::MessageRing;

pub struct MessagesView {
    state: ViewState,
    ring: Arc<Mutex<MessageRing>>,
    scroll: usize,
    last_gen: u64,
}

impl MessagesView {
    pub fn new(ring: Arc<Mutex<MessageRing>>) -> Self {
        Self {
            state: ViewState::default(),
            ring,
            scroll: 0,
            last_gen: 0,
        }
    }
}

impl View for MessagesView {
    delegate_view_state!(state, override { title });

    fn title(&self) -> &str {
        "Messages"
    }

    fn draw(&self, surface: &mut Surface) {
        let b = self.state.bounds();
        if b.w == 0 || b.h == 0 {
            return;
        }
        let ring = match self.ring.lock() {
            Ok(r) => r,
            Err(_) => return,
        };
        let entries = ring.entries();
        let rows = b.h as usize;
        let total = entries.len();
        let start = if total > rows + self.scroll {
            total - rows - self.scroll
        } else {
            0
        };
        for row in 0..rows {
            let y = b.y + row as u16;
            if let Some(msg) = entries.get(start + row) {
                // Local time from epoch via libc
                let t = msg.timestamp as i64;
                let (hrs, mins, secs) = epoch_to_local_hms(t);
                let suffix = if msg.count > 1 {
                    format!(" (×{})", msg.count)
                } else {
                    String::new()
                };
                let line = format!(
                    "[{hrs:02}:{mins:02}:{secs:02}] [{:>4}] [{}] {}{}",
                    msg.level.label(),
                    msg.origin,
                    msg.text,
                    suffix,
                );
                let style = match msg.level {
                    MsgLevel::Error => Style {
                        fg: Color::Ansi(9),
                        ..Style::default()
                    },
                    MsgLevel::Warn => Style {
                        fg: Color::Ansi(11),
                        ..Style::default()
                    },
                    MsgLevel::Debug => Style {
                        fg: Color::Ansi(8),
                        ..Style::default()
                    },
                    MsgLevel::Info => Style::default(),
                };
                surface.print_line(b.x, y, &line, b.w, style);
            } else {
                surface.print_line(b.x, y, "", b.w, Style::default());
            }
        }
    }

    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    let total = self.ring.lock().map(|r| r.len()).unwrap_or(0);
                    if self.scroll < total {
                        self.scroll += 1;
                    }
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.scroll > 0 {
                        self.scroll -= 1;
                    }
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::Home | KeyCode::Char('g') => {
                    let total = self.ring.lock().map(|r| r.len()).unwrap_or(0);
                    self.scroll = total.saturating_sub(1);
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                KeyCode::End | KeyCode::Char('G') => {
                    self.scroll = 0;
                    self.state.mark_dirty();
                    HandleResult::Consumed
                }
                _ => HandleResult::Ignored,
            },
            Event::Tick => {
                let gen = self.ring.lock().map(|r| r.generation()).unwrap_or(0);
                if gen != self.last_gen {
                    self.last_gen = gen;
                    self.state.mark_dirty();
                }
                HandleResult::Ignored
            }
            _ => HandleResult::Ignored,
        }
    }
}

fn epoch_to_local_hms(epoch: i64) -> (u64, u64, u64) {
    #[cfg(unix)]
    {
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        unsafe { libc::localtime_r(&epoch, &mut tm) };
        (tm.tm_hour as u64, tm.tm_min as u64, tm.tm_sec as u64)
    }
    #[cfg(not(unix))]
    {
        let hrs = ((epoch / 3600) % 24) as u64;
        let mins = ((epoch / 60) % 60) as u64;
        let secs = (epoch % 60) as u64;
        (hrs, mins, secs)
    }
}
