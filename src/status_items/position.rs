//! CtxPositionItem — displays cursor position in status bar.

use txv_core::prelude::*;
use txv_widgets::CursorPos;

use super::helpers::{draw_label, indicator_state, sync_bounds};
use crate::commands::{ViewContext, CM_CONTEXT_UPDATE, CM_CURSOR_MOVED};

/// Displays cursor position: "Ln N, Col M".
pub struct CtxPositionItem {
    state: ViewState,
    label: String,
}

impl Default for CtxPositionItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxPositionItem {
    pub fn new() -> Self {
        let label = "~Ln~ 1, ~Col~ 1".to_string();
        Self {
            state: indicator_state(label.len() as u16 + 2),
            label,
        }
    }
}

impl View for CtxPositionItem {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        draw_label(&mut self.state, &self.label);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        let Event::Command { id, data, .. } = event else {
            return HandleResult::Ignored;
        };
        if *id == CM_CURSOR_MOVED {
            if let Some(pos) = data.as_ref().and_then(|d| d.downcast_ref::<CursorPos>()) {
                self.label = format!("~Ln~ {}, ~Col~ {}", pos.line(), pos.col());
                sync_bounds(&mut self.state, &self.label);
                self.state.mark_dirty();
            }
        }
        if *id == CM_CONTEXT_UPDATE {
            if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                if vc.line > 0 {
                    self.label = format!("~Ln~ {}, ~Col~ {}", vc.line, vc.col);
                    sync_bounds(&mut self.state, &self.label);
                    self.state.mark_dirty();
                }
            }
        }
        HandleResult::Ignored
    }
}
