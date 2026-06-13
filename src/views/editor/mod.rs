//! EditorView — kairn's editor backed by txv-edit's EditorView<KairnDelegate>.
//!
//! EditorView wraps txv_edit::view::EditorView with KairnDelegate providing:
//! - Git gutter signs
//! - Diagnostics underlines
//! - Blame gutter
//! - LSP integration (completion via DropdownMenu, signature, goto)
//! - Autosave, disk change detection
//!
//! Diff mode is a SEPARATE DiffView component, not part of EditorView.

mod build;
mod delegate;
mod delegate_accessors;
mod delegate_diff;
pub mod diff_model;
pub mod diff_opts;
mod handle_action;
mod handle_command_event;
mod handle_completion;
mod handle_signature;
mod handle_tick;
mod methods;
mod methods_diff;
pub mod sbs_model;

use std::mem;

use txv_core::prelude::*;

use crate::buffer_registry::BufferId;

pub use delegate::KairnDelegate;

/// Kairn's editor view — wraps txv-edit's EditorView with IDE extensions.
pub struct EditorView {
    pub(crate) inner: txv_edit::view::EditorView<KairnDelegate>,
    /// Buffer identity in the shared registry (assigned on open).
    pub(crate) buffer_id: Option<BufferId>,
}

impl std::ops::Deref for EditorView {
    type Target = txv_edit::view::EditorView<KairnDelegate>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl std::ops::DerefMut for EditorView {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl View for EditorView {
    fn view_id(&self) -> ViewId {
        self.inner.view_id()
    }
    fn bounds(&self) -> Rect {
        self.inner.bounds()
    }
    fn set_bounds(&mut self, r: Rect) {
        self.inner.set_bounds(r);
        // Ensure cursor stays visible after viewport resize
        let line = self.editor().cursor_line();
        self.scroll_to_line(line);
    }
    fn set_sink(&mut self, sink: txv_core::view::EventSink) {
        self.inner.set_sink(sink);
    }
    fn options(&self) -> ViewOptions {
        self.inner.options()
    }
    fn title(&self) -> &str {
        self.inner.title()
    }
    fn needs_redraw(&self) -> bool {
        self.inner.needs_redraw()
    }
    fn mark_redrawn(&mut self) {
        self.inner.mark_redrawn();
    }
    fn select(&mut self) {
        self.inner.select();
    }
    fn unselect(&mut self) {
        self.inner.unselect();
    }
    fn cursor(&self) -> Option<CursorRequest> {
        self.inner.cursor()
    }
    fn buffer(&self) -> &Buffer {
        self.inner.buffer()
    }
    fn can_close(&self) -> CloseResult {
        self.inner.can_close()
    }
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
    fn render(&mut self) -> bool {
        // Sync before render
        let num = self.inner.editor().options().number();
        self.inner.delegate_mut().settings_mut().set_number(num);
        let rendered = self.inner.render();
        // Post-render: draw completion popup
        if rendered && self.inner.delegate().completion_visible() {
            let d = self.inner.delegate_mut();
            let popup = mem::take(&mut d.completion_popup);
            popup.draw(self.inner.buffer_mut());
            let d2 = self.inner.delegate_mut();
            d2.completion_popup = popup;
        }
        rendered
    }
    fn draw(&mut self) {
        let num = self.inner.editor().options().number();
        self.inner.delegate_mut().settings_mut().set_number(num);
        self.inner.draw();
    }
    fn handle(&mut self, event: &Event) -> HandleResult {
        let old_mode = self.inner.editor().mode();
        let result = self.inner.handle(event);
        self.flush_pending();
        let new_mode = self.inner.editor().mode();
        if new_mode != old_mode {
            use crate::commands::CM_MODE_CHANGED;
            use crate::editor::keymap::EditorMode;
            let is_cmdline_transition = matches!(old_mode, EditorMode::Command | EditorMode::Search)
                || matches!(new_mode, EditorMode::Command | EditorMode::Search);
            if is_cmdline_transition {
                let name = match new_mode {
                    EditorMode::Normal => "NOR",
                    EditorMode::Insert => "INS",
                    EditorMode::Visual | EditorMode::VisualLine | EditorMode::VisualBlock => "VIS",
                    EditorMode::Command | EditorMode::Search => "CMD",
                };
                self.inner
                    .put_command(CM_MODE_CHANGED, Some(Box::new(name.to_string())));
            }
        }
        result
    }
}
