//! Status bar items that read from CM_CONTEXT_UPDATE broadcasts.

use txv_core::prelude::*;
use txv_core::status::{ActiveItem, Gravity, VisibleItem};
use txv_widgets::CursorPos;

use crate::commands::{ViewContext, CM_CONTEXT_UPDATE, CM_CURSOR_MOVED, CM_MODE_CHANGED};

/// Displays cursor position: "Ln N, Col M".
pub struct CtxPositionItem {
    label: String,
}

impl Default for CtxPositionItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxPositionItem {
    pub fn new() -> Self {
        Self {
            label: "Ln 1, Col 1".into(),
        }
    }
}

impl ActiveItem for CtxPositionItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_CURSOR_MOVED {
                if let Some(pos) = data.as_ref().and_then(|d| d.downcast_ref::<CursorPos>()) {
                    self.label = format!("Ln {}, Col {}", pos.line, pos.col);
                    return HandleResult::Consumed;
                }
            }
            if *id == CM_CONTEXT_UPDATE {
                if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                    if vc.line > 0 {
                        self.label = format!("Ln {}, Col {}", vc.line, vc.col);
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for CtxPositionItem {
    fn label(&self) -> &str {
        &self.label
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
}

/// Displays editor mode: "NOR", "INS", "VIS", etc.
pub struct CtxModeItem {
    label: String,
}

impl Default for CtxModeItem {
    fn default() -> Self {
        Self::new()
    }
}

impl CtxModeItem {
    pub fn new() -> Self {
        Self { label: "NOR".into() }
    }
}

impl ActiveItem for CtxModeItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_MODE_CHANGED {
                if let Some(mode) = data.as_ref().and_then(|d| d.downcast_ref::<String>()) {
                    self.label = mode.clone();
                    return HandleResult::Consumed;
                }
            }
            if *id == CM_CONTEXT_UPDATE {
                if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                    if !vc.mode.is_empty() {
                        self.label = vc.mode.clone();
                    }
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for CtxModeItem {
    fn label(&self) -> &str {
        &self.label
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
}

/// Displays file language: "rust", "go", etc.
#[derive(Default)]
pub struct CtxLangItem {
    label: String,
}

impl CtxLangItem {
    pub fn new() -> Self {
        Self { label: String::new() }
    }
}

impl ActiveItem for CtxLangItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_CONTEXT_UPDATE {
                if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                    self.label = vc.language.clone();
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for CtxLangItem {
    fn label(&self) -> &str {
        &self.label
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
}

/// Displays "[+]" when buffer is modified.
#[derive(Default)]
pub struct CtxModifiedItem {
    label: String,
}

impl CtxModifiedItem {
    pub fn new() -> Self {
        Self { label: String::new() }
    }
}

impl ActiveItem for CtxModifiedItem {
    fn handle(&mut self, event: &Event, _sink: &EventSink) -> HandleResult {
        if let Event::Command { id, data } = event {
            if *id == CM_CONTEXT_UPDATE {
                if let Some(vc) = data.as_ref().and_then(|d| d.downcast_ref::<ViewContext>()) {
                    self.label = if vc.modified {
                        "[+]".into()
                    } else {
                        String::new()
                    };
                    return HandleResult::Consumed;
                }
            }
        }
        HandleResult::Ignored
    }
}

impl VisibleItem for CtxModifiedItem {
    fn label(&self) -> &str {
        &self.label
    }
    fn gravity(&self) -> Gravity {
        Gravity::Right
    }
}
