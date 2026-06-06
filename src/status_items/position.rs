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
        let label = "1,1 Top".to_string();
        Self {
            state: indicator_state(label.len() as u16 + 2),
            label,
        }
    }

    fn scroll_pct(line: u32, total: u32) -> String {
        if total <= 1 {
            "All".to_string()
        } else if line <= 1 {
            "Top".to_string()
        } else if line >= total {
            "Bot".to_string()
        } else {
            format!("{}%", (line - 1) * 100 / (total - 1))
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
                self.label = format!("{},{}", pos.line(), pos.col());
                sync_bounds(&mut self.state, &self.label);
                self.state.mark_dirty();
            }
        }
        if *id == CM_CONTEXT_UPDATE {
            if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                if vc.line > 0 {
                    let pct = Self::scroll_pct(vc.line, vc.total_lines);
                    self.label = format!("{},{} {}", vc.line, vc.col, pct);
                    sync_bounds(&mut self.state, &self.label);
                    self.state.mark_dirty();
                }
            }
        }
        HandleResult::Ignored
    }
}
