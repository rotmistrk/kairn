//! Prompt mode handling for KairnStatusBar.

use txv_core::prelude::*;
use crate::commands::*;
use super::KairnStatusBar;

impl KairnStatusBar {
    pub(super) fn enter_prompt(&mut self) {
        self.mode = super::Mode::Prompt;
        self.input.clear();
        let b = self.inner.bounds();
        if b.w > 1 { self.input.set_bounds(Rect::new(b.x + 1, b.y, b.w - 1, 1)); }
    }

    pub(super) fn exit_prompt(&mut self) {
        self.mode = super::Mode::Normal;
        self.input.clear();
    }

    pub(super) fn try_complete(&mut self) {
        if let Some(ref completer) = self.completer {
            let completions = completer.complete(&self.input.text, self.input.cursor);
            if completions.len() == 1 {
                self.input.set_text(&completions[0].text);
            } else if !completions.is_empty() {
                self.input.completions = completions.iter().map(|c| c.display.clone()).collect();
            }
        }
    }

    pub(super) fn handle_prompt(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else { return HandleResult::Ignored; };
        if key.code == KeyCode::Tab {
            self.try_complete();
            return HandleResult::Consumed;
        }
        let result = self.input.handle(event, queue);
        let events = queue.drain();
        for ev in events {
            if let Event::Command { id, data } = &ev {
                if *id == CM_OK {
                    if let Some(boxed) = data.as_ref() {
                        if let Some(text) = boxed.downcast_ref::<String>() {
                            let cmd_text = text.clone();
                            self.exit_prompt();
                            queue.put_command(CM_EXECUTE_COMMAND, Some(Box::new(cmd_text)));
                            return HandleResult::Consumed;
                        }
                    }
                } else if *id == CM_CANCEL {
                    self.exit_prompt();
                    return HandleResult::Consumed;
                }
            }
            queue.put(ev);
        }
        result
    }
}
