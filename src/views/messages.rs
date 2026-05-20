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

    fn draw(&mut self) {
        let w = self.state.buffer_mut().width();
        let h = self.state.buffer_mut().height();
        if w == 0 || h == 0 {
            return;
        }
        let ring = match self.ring.lock() {
            Ok(r) => r,
            Err(_) => return,
        };
        let entries = ring.entries();
        let rows = h as usize;
        let total = entries.len();
        let start = if total > rows + self.scroll {
            total - rows - self.scroll
        } else {
            0
        };
        let lines: Vec<(String, Style)> = (0..rows)
            .map(|row| {
                if let Some(msg) = entries.get(start + row) {
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
                    let app = crate::app_palette::app_palette();
                    let style = match msg.level {
                        MsgLevel::Error => app.msg.error.to_style(),
                        MsgLevel::Warn => app.msg.warning.to_style(),
                        MsgLevel::Debug => app.msg.debug.to_style(),
                        MsgLevel::Info => app.msg.info.to_style(),
                    };
                    (line, style)
                } else {
                    (String::new(), Style::default())
                }
            })
            .collect();
        drop(ring);
        for (row, (line, style)) in lines.iter().enumerate() {
            self.state.buffer_mut().print_line(0, row as u16, line, w, *style);
        }
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
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
