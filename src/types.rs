//! Kairn-specific types for view communication.

use txv_widgets::view::CommandId;

/// Identifies a slot in the SlottedDesktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotId {
    Left,
    Center,
    Right,
    Bottom,
}

/// Payload for CM_OPEN_FILE command.
#[derive(Debug, Clone)]
pub struct OpenFilePayload {
    pub path: String,
}

/// A pending command with optional payload.
pub struct PendingCommand {
    pub id: CommandId,
    pub payload: Option<Box<dyn std::any::Any + Send>>,
}

/// Outbox for views to emit commands.
#[derive(Default)]
pub struct CommandOutbox {
    pub commands: Vec<PendingCommand>,
}

impl CommandOutbox {
    pub fn emit(&mut self, cmd: CommandId) {
        self.commands.push(PendingCommand { id: cmd, payload: None });
    }

    pub fn emit_with<T: std::any::Any + Send + 'static>(&mut self, cmd: CommandId, payload: T) {
        self.commands.push(PendingCommand { id: cmd, payload: Some(Box::new(payload)) });
    }

    pub fn drain(&mut self) -> Vec<PendingCommand> {
        std::mem::take(&mut self.commands)
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}
