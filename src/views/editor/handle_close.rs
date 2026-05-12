//! EditorView close prompt handling (save/discard/cancel on close).

use txv_core::prelude::*;

use super::EditorView;
use crate::commands::*;

impl EditorView {
    pub(super) fn handle_close_prompt(
        &mut self,
        key: &txv_core::event::KeyEvent,
        queue: &mut EventQueue,
    ) -> HandleResult {
        use txv_core::event::KeyCode;
        match &key.code {
            KeyCode::Char('y') => {
                self.close_prompt = false;
                let content = self.editor.buffer.content();
                match crate::editor::save::save_file(&self.path, &content) {
                    Ok(()) => {
                        self.editor.buffer.mark_saved();
                        queue.put_command(CM_FILE_CLOSED, Some(Box::new(self.path.to_string_lossy().to_string())));
                        queue.put_command(CM_TAB_CLOSE, None);
                    }
                    Err(e) => {
                        let msg = txv_core::message::Message::error("editor", format!("Save failed: {e}"));
                        queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                    }
                }
            }
            KeyCode::Char('n') => {
                self.close_prompt = false;
                self.editor.buffer.mark_saved();
                queue.put_command(CM_FILE_CLOSED, Some(Box::new(self.path.to_string_lossy().to_string())));
                queue.put_command(CM_TAB_CLOSE, None);
            }
            _ => {
                self.close_prompt = false;
                self.editor.status = String::new();
            }
        }
        self.state.mark_dirty();
        HandleResult::Consumed
    }
}
