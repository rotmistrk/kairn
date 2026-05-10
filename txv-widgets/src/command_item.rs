//! CommandItem — status bar item for command input with completion.

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};

pub struct CommandItem {
    activation_keys: Vec<KeyEvent>,
    command_id: CommandId,
    active: bool,
    text: String,
    cursor: usize,
    completer: Option<Box<dyn Completer>>,
    label_text: String,
    dormant_label: String,
    gravity: Gravity,
}

impl CommandItem {
    pub fn new(keys: &[KeyEvent], command_id: CommandId) -> Self {
        Self {
            activation_keys: keys.to_vec(),
            active: false,
            command_id,
            text: String::new(),
            cursor: 0,
            completer: None,
            label_text: String::new(),
            dormant_label: String::new(),
            gravity: Gravity::Left,
        }
    }
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.dormant_label = label.into();
        self.label_text = self.dormant_label.clone();
        self
    }
    pub fn with_completer(mut self, c: Box<dyn Completer>) -> Self {
        self.completer = Some(c);
        self
    }
    pub fn set_completer(&mut self, c: Box<dyn Completer>) {
        self.completer = Some(c);
    }

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
    fn update_label(&mut self) {
        self.label_text = format!(":{}", self.text);
    }

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
                if self.activation_keys.contains(k) {
                    self.activate();
                    return HandleResult::Consumed;
                }
            }
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return HandleResult::Consumed;
        };
        match &key.code {
            KeyCode::Esc => self.deactivate(),
            KeyCode::Enter => {
                let cmd = self.text.clone();
                self.deactivate();
                if !cmd.is_empty() {
                    queue.put_command(self.command_id, Some(Box::new(cmd)));
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
            KeyCode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            KeyCode::Right => {
                if self.cursor < self.text.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Char(ch) => {
                self.text.insert(self.cursor, *ch);
                self.cursor += 1;
                self.update_label();
            }
            _ => {}
        }
        HandleResult::Consumed
    }
    fn is_exclusive(&self) -> bool {
        self.active
    }
}

impl VisibleItem for CommandItem {
    fn label(&self) -> &str {
        &self.label_text
    }
    fn gravity(&self) -> Gravity {
        self.gravity
    }
}
