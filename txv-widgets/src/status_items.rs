//! StatusBar item implementations: KeyLabel, Clock, Command, Message.

use std::time::Instant;
use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Command ID for setting status message externally.
pub const CM_STATUS_MESSAGE: CommandId = 140;

// --- KeyLabelItem ---

pub struct KeyLabelItem {
    key: KeyEvent,
    command: CommandId,
    label_text: String,
    gravity: Gravity,
}

impl KeyLabelItem {
    pub fn new(key: KeyEvent, command: CommandId, label: impl Into<String>) -> Self {
        Self {
            key,
            command,
            label_text: label.into(),
            gravity: Gravity::Left,
        }
    }
    pub fn hidden(key: KeyEvent, command: CommandId) -> Self {
        Self {
            key,
            command,
            label_text: String::new(),
            gravity: Gravity::Left,
        }
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }
}

impl ActiveItem for KeyLabelItem {
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(k) = event {
            if *k == self.key {
                queue.put_command(self.command, None);
                return HandleResult::Consumed;
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for KeyLabelItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
}

// --- ClockItem ---

pub struct ClockItem {
    interval: u16,
    last_update: Instant,
    label_text: String,
    gravity: Gravity,
}

impl ClockItem {
    pub fn new(interval: u16) -> Self {
        let mut item = Self {
            interval,
            last_update: Instant::now(),
            label_text: String::new(),
            gravity: Gravity::Right,
        };
        item.refresh_time();
        item
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }

    fn refresh_time(&mut self) {
        let (h, m) = local_hm();
        self.label_text = format!("{h:02}:{m:02}");
        self.last_update = Instant::now();
    }
}

impl VisibleItem for ClockItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
    fn tick(&mut self) {
        if self.interval > 0 && self.last_update.elapsed().as_secs() >= u64::from(self.interval) {
            self.refresh_time();
        }
    }
}

fn local_hm() -> (u32, u32) {
    use std::time::SystemTime;
    let secs = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as libc::time_t;
    let mut tm: libc::tm = unsafe { std::mem::zeroed() };
    unsafe { libc::localtime_r(&secs, &mut tm) };
    (tm.tm_hour as u32, tm.tm_min as u32)
}

// --- MessageItem ---

pub struct MessageItem {
    display: String,
    style: Style,
    timeout_secs: u16,
    last_set: Option<Instant>,
    gravity: Gravity,
}

impl MessageItem {
    pub fn new(timeout_secs: u16) -> Self {
        Self {
            display: String::new(),
            style: Style::default(),
            timeout_secs,
            last_set: None,
            gravity: Gravity::Right,
        }
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self {
        self.gravity = g;
        self
    }
}

impl ActiveItem for MessageItem {
    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_STATUS_MESSAGE {
                if let Some(boxed) = data.as_ref() {
                    if let Some(msg) = boxed.downcast_ref::<Message>() {
                        if msg.level != MsgLevel::Debug {
                            self.display = format!("[{}] {}", msg.origin, msg.text);
                            self.style = Style {
                                fg: match msg.level {
                                    MsgLevel::Error => Color::Ansi(9),
                                    MsgLevel::Warn => Color::Ansi(11),
                                    _ => Color::Ansi(7),
                                },
                                ..Style::default()
                            };
                            self.last_set = Some(Instant::now());
                        }
                        // Don't consume — let handler append to ring
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for MessageItem {
    fn label(&self) -> &str {
        &self.display
    }
    fn style(&self) -> Style {
        self.style
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
    fn tick(&mut self) {
        if self.timeout_secs == 0 {
            return;
        }
        if let Some(set_at) = self.last_set {
            if set_at.elapsed().as_secs() >= u64::from(self.timeout_secs) {
                self.display.clear();
                self.last_set = None;
            }
        }
    }
}
