//! CommandStore — emits a command with content string (for virtual buffers).

use txv_core::prelude::EventSink;

use crate::buffer_store::BufferStore;

/// Emits a command with the content string (for virtual buffers like todo notes).
pub struct CommandStore {
    command_id: u16,
    sink: EventSink,
}

impl CommandStore {
    pub fn new(command_id: u16, sink: EventSink) -> Self {
        Self { command_id, sink }
    }
}

impl BufferStore for CommandStore {
    fn save(&mut self, content: &str) -> Result<(), String> {
        self.sink
            .push_command(self.command_id, Some(Box::new(content.to_string())));
        Ok(())
    }
}
