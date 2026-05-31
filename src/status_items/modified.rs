//! CtxModifiedItem — displays "[+]" when buffer is modified.

use txv_core::prelude::*;

use super::helpers::{draw_label, indicator_state, sync_bounds};
use crate::commands::{ViewContext, CM_CONTEXT_UPDATE};

/// Displays "[+]" when buffer is modified.
#[derive(Default)]
pub struct CtxModifiedItem {
    state: ViewState,
    label: String,
}

impl CtxModifiedItem {
    pub fn new() -> Self {
        Self {
            state: indicator_state(0),
            label: String::new(),
        }
    }
}

impl View for CtxModifiedItem {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        draw_label(&mut self.state, &self.label);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id == CM_CONTEXT_UPDATE {
            if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                self.label = if vc.modified {
                    "[+]".into()
                } else {
                    String::new()
                };
                sync_bounds(&mut self.state, &self.label);
                self.state.mark_dirty();
            }
        }
        HandleResult::Ignored
    }
}
