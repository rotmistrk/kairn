//! View trait implementation for StructuredView.

use txv_core::prelude::*;

use crate::commands::CM_SAVE;

use super::handle;
use super::StructuredView;

impl View for StructuredView {
    fn view_id(&self) -> txv_core::view::ViewId {
        self.tree.state.id()
    }

    fn bounds(&self) -> Rect {
        self.tree.state.bounds()
    }

    fn set_bounds(&mut self, r: Rect) {
        if self.tree.state.bounds() != r {
            self.cancel_edit();
            self.filtering = false;
            self.sort_path_target = None;
        }
        self.tree.state.set_bounds(r);
        self.update_col_widths();
    }

    fn set_sink(&mut self, sink: EventSink) {
        self.tree.state.set_sink(sink);
    }

    fn options(&self) -> txv_core::view::ViewOptions {
        self.tree.state.options()
    }

    fn title(&self) -> &str {
        &self.display_title
    }

    fn needs_redraw(&self) -> bool {
        self.tree.state.is_dirty()
    }

    fn mark_redrawn(&mut self) {
        self.tree.state.mark_redrawn();
    }

    fn select(&mut self) {
        self.tree.select();
    }

    fn unselect(&mut self) {
        self.tree.unselect();
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn buffer(&self) -> &txv_core::buffer::Buffer {
        self.tree.state.buffer()
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if self.is_editing() {
            if let Some(input) = &self.input_line {
                return input.cursor();
            }
        }
        None
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }

    fn draw(&mut self) {
        self.tree.draw();
        self.blit_input_line();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, .. } = event {
            if *id == CM_SAVE {
                return handle::handle_save_command(self);
            }
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if self.is_editing() {
            if let Some(input) = self.input_line.as_mut() {
                input.handle(event);
            }
            handle::drain_edit_commands(self);
            self.tree.state.mark_dirty();
            return HandleResult::Consumed;
        }
        handle::handle_struct_key(self, key)
    }
}
