//! CtxModeItem — displays editor mode in status bar.

use txv_core::prelude::*;

use super::helpers::{draw_label, indicator_state, sync_bounds};
use crate::commands::{ViewContext, CM_CONTEXT_UPDATE, CM_MODE_CHANGED};

/// Displays editor mode: "NOR", "INS", "VIS", etc.
pub struct CtxModeItem {
    state: ViewState,
    label: String,
}

impl Default for CtxModeItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxModeItem {
    pub fn new() -> Self {
        let label = "NOR".to_string();
        Self {
            state: indicator_state(label.len() as u16 + 2),
            label,
        }
    }
}

impl View for CtxModeItem {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        draw_label(&mut self.state, &self.label);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id == CM_MODE_CHANGED {
            if let Some(mode) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                self.label = mode.clone();
                sync_bounds(&mut self.state, &self.label);
                self.state.mark_dirty();
            }
        }
        if *id == CM_CONTEXT_UPDATE {
            if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                if !vc.mode.is_empty() {
                    self.label = vc.mode.clone();
                    sync_bounds(&mut self.state, &self.label);
                    self.state.mark_dirty();
                }
            }
        }
        HandleResult::Ignored
    }
}
