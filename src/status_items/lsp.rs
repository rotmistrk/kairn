//! LSP status indicator for the status bar.

use txv_core::palette::{palette, StyleId};
use txv_core::prelude::*;

use crate::commands::CM_LSP_STATUS_UPDATE;
use crate::lsp::progress::{format_status_label, LspServerState};

/// Displays per-language LSP server state: "rust ✓ go ⟳"
pub struct LspStatusItem {
    state: ViewState,
    label: String,
}

impl Default for LspStatusItem {
    fn default() -> Self {
        Self::new()
    }
}

impl LspStatusItem {
    pub fn new() -> Self {
        let mut state = ViewState::new(ViewOptions {
            preprocess: true,
            focusable: false,
            ..ViewOptions::default()
        });
        state.set_bounds(Rect { x: 0, y: 0, w: 0, h: 1 });
        Self {
            state,
            label: String::new(),
        }
    }

    fn sync_bounds(&mut self) {
        let w = if self.label.is_empty() {
            0
        } else {
            self.label.len() as u16 + 2
        };
        let b = self.state.bounds();
        if b.w != w {
            self.state.set_bounds(Rect {
                x: b.x,
                y: b.y,
                w,
                h: 1,
            });
        }
    }
}

impl View for LspStatusItem {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        let style = palette().style(StyleId::StatusBar);
        self.state.buffer_mut().fill(' ', style);
        if !self.label.is_empty() {
            self.state.buffer_mut().print(1, 0, &self.label, style);
        }
        self.state.mark_redrawn();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data, .. } = event {
            if *id == CM_LSP_STATUS_UPDATE {
                if let Some(snapshot) = data
                    .as_ref()
                    .and_then(|d| d.downcast_ref::<Vec<(String, LspServerState, Option<u64>)>>())
                {
                    self.label = format_status_label(snapshot);
                    self.sync_bounds();
                    self.state.mark_dirty();
                }
            }
        }
        HandleResult::Ignored
    }
}
