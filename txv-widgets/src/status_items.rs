//! StatusBar item implementations: KeyLabel, Clock, Command, Message.

use std::time::Instant;
use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

/// Command ID used by CommandItem to emit executed commands.
const CM_EXECUTE_COMMAND: CommandId = 131;
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
        Self { key, command, label_text: label.into(), gravity: Gravity::Left }
    }
    pub fn hidden(key: KeyEvent, command: CommandId) -> Self {
        Self { key, command, label_text: String::new(), gravity: Gravity::Left }
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self { self.gravity = g; self }
}

impl ActiveItem for KeyLabelItem {
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(k) = event {
            if *k == self.key { queue.put_command(self.command, None); return HandleResult::Consumed; }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for KeyLabelItem {
    fn label(&self) -> &str { &self.label_text }
    fn gravity(&self) -> Gravity { self.gravity }
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
            interval, last_update: Instant::now(), label_text: String::new(), gravity: Gravity::Right,
        };
        item.refresh_time();
        item
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self { self.gravity = g; self }

    fn refresh_time(&mut self) {
        let (h, m) = local_hm();
        self.label_text = format!("{h:02}:{m:02}");
        self.last_update = Instant::now();
    }
}

impl VisibleItem for ClockItem {
    fn label(&self) -> &str { &self.label_text }
    fn gravity(&self) -> Gravity { self.gravity }
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

// --- CommandItem ---

pub struct CommandItem {
    activation_keys: Vec<KeyEvent>,
    active: bool,
    text: String,
    cursor: usize,
    completer: Option<Box<dyn Completer>>,
    label_text: String,
    dormant_label: String,
    gravity: Gravity,
}

impl CommandItem {
    pub fn new(keys: &[KeyEvent]) -> Self {
        Self {
            activation_keys: keys.to_vec(), active: false, text: String::new(),
            cursor: 0, completer: None, label_text: String::new(),
            dormant_label: String::new(), gravity: Gravity::Left,
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.dormant_label = label.into();
        self.label_text = self.dormant_label.clone();
        self
    }
    pub fn with_completer(mut self, c: Box<dyn Completer>) -> Self { self.completer = Some(c); self }
    pub fn set_completer(&mut self, c: Box<dyn Completer>) { self.completer = Some(c); }

    fn activate(&mut self) {
        self.active = true;
        self.text.clear();
        self.cursor = 0;
        self.label_text = ":".to_string();
    }
    fn deactivate(&mut self) {
        self.active = false;
        self.text.clear();
        self.cursor = 0;
        self.label_text = self.dormant_label.clone();
    }
    fn update_label(&mut self) { self.label_text = format!(":{}", self.text); }

    fn try_complete(&mut self) {
        if let Some(ref completer) = self.completer {
            let completions = completer.complete(&self.text, self.cursor);
            if completions.len() == 1 {
                self.text = completions[0].text.clone();
                self.cursor = self.text.len();
                self.update_label();
            }
        }
    }
}

impl ActiveItem for CommandItem {
    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if !self.active {
            if let Event::Key(k) = event {
                if self.activation_keys.contains(k) { self.activate(); return HandleResult::Consumed; }
            }
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else { return HandleResult::Consumed; };
        match &key.code {
            KeyCode::Esc => self.deactivate(),
            KeyCode::Enter => {
                let cmd = self.text.clone();
                self.deactivate();
                if !cmd.is_empty() {
                    queue.put_command(CM_EXECUTE_COMMAND, Some(Box::new(cmd)));
                }
            }
            KeyCode::Tab => self.try_complete(),
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                    self.update_label();
                } else {
                    self.deactivate();
                }
            }
            KeyCode::Left => { if self.cursor > 0 { self.cursor -= 1; } }
            KeyCode::Right => { if self.cursor < self.text.len() { self.cursor += 1; } }
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += 1;
                self.update_label();
            }
            _ => {}
        }
        HandleResult::Consumed
    }
    fn is_exclusive(&self) -> bool { self.active }
}

impl VisibleItem for CommandItem {
    fn label(&self) -> &str { &self.label_text }
    fn gravity(&self) -> Gravity { self.gravity }
}

// --- MessageItem ---

pub struct MessageItem {
    message: String,
    timeout_secs: u16,
    last_set: Option<Instant>,
    gravity: Gravity,
}

impl MessageItem {
    pub fn new(timeout_secs: u16) -> Self {
        Self { message: String::new(), timeout_secs, last_set: None, gravity: Gravity::Right }
    }
    pub fn with_gravity(mut self, g: Gravity) -> Self { self.gravity = g; self }
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        self.last_set = Some(Instant::now());
    }
}

impl ActiveItem for MessageItem {
    fn handle(&mut self, event: &Event, _queue: &mut EventQueue) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_STATUS_MESSAGE {
                if let Some(boxed) = data.as_ref() {
                    if let Some(msg) = boxed.downcast_ref::<String>() {
                        self.set_message(msg.clone());
                        return HandleResult::Consumed;
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for MessageItem {
    fn label(&self) -> &str { &self.message }
    fn gravity(&self) -> Gravity { self.gravity }
    fn tick(&mut self) {
        if self.timeout_secs == 0 { return; }
        if let Some(set_at) = self.last_set {
            if set_at.elapsed().as_secs() >= u64::from(self.timeout_secs) {
                self.message.clear();
                self.last_set = None;
            }
        }
    }
}
