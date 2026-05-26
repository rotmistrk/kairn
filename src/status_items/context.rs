//! Context-aware status bar indicators (position, mode, language, modified).

use txv_core::prelude::*;
use txv_widgets::CursorPos;

use crate::commands::{ViewContext, CM_CONTEXT_UPDATE, CM_CURSOR_MOVED, CM_MODE_CHANGED};

/// Helper: create ViewState for a status indicator.
fn indicator_state(initial_width: u16) -> ViewState {
    let mut state = ViewState::new(ViewOptions {
        preprocess: true,
        focusable: false,
        ..ViewOptions::default()
    });
    state.set_bounds(Rect {
        x: 0,
        y: 0,
        w: initial_width,
        h: 1,
    });
    state
}

/// Helper: update bounds to match label width.
fn sync_bounds(state: &mut ViewState, label: &str) {
    let w = if label.is_empty() {
        0
    } else {
        label.len() as u16 + 2
    };
    let b = state.bounds();
    if b.w != w {
        state.set_bounds(Rect {
            x: b.x,
            y: b.y,
            w,
            h: 1,
        });
    }
}

/// Helper: draw label into buffer with reversed style.
fn draw_label(state: &mut ViewState, label: &str) {
    let style = Style {
        attrs: Attrs {
            reverse: true,
            ..Attrs::default()
        },
        ..Style::default()
    };
    state.buffer_mut().fill(' ', style);
    if !label.is_empty() {
        state.buffer_mut().print(1, 0, label, style);
    }
    state.mark_redrawn();
}

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
        let label = "Ln 1, Col 1".to_string();
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
        if let Event::Command { id, data } = event {
            if *id == CM_CURSOR_MOVED {
                if let Some(pos) = data.as_ref().and_then(|d| d.downcast_ref::<CursorPos>()) {
                    self.label = format!("Ln {}, Col {}", pos.line, pos.col);
                    sync_bounds(&mut self.state, &self.label);
                    self.state.mark_dirty();
                }
            }
            if *id == CM_CONTEXT_UPDATE {
                if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                    if vc.line > 0 {
                        self.label = format!("Ln {}, Col {}", vc.line, vc.col);
                        sync_bounds(&mut self.state, &self.label);
                        self.state.mark_dirty();
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

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
        if let Event::Command { id, data } = event {
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
        }
        HandleResult::Ignored
    }
}

/// Displays file language: "rust", "go", etc.
#[derive(Default)]
pub struct CtxLangItem {
    state: ViewState,
    label: String,
}

impl CtxLangItem {
    pub fn new() -> Self {
        Self {
            state: indicator_state(0),
            label: String::new(),
        }
    }
}

impl View for CtxLangItem {
    delegate_view_state!(state, override { draw, handle });

    fn draw(&mut self) {
        draw_label(&mut self.state, &self.label);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_CONTEXT_UPDATE {
                if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                    self.label = vc.language.clone();
                    sync_bounds(&mut self.state, &self.label);
                    self.state.mark_dirty();
                }
            }
        }
        HandleResult::Ignored
    }
}

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
        if let Event::Command { id, data } = event {
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
        }
        HandleResult::Ignored
    }
}
