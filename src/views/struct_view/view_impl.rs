//! View trait implementation for StructuredView.

use txv_core::prelude::*;

use crate::commands::CM_FS_CHANGED;

use super::handle;
use super::StructuredView;

impl View for StructuredView {
    delegate_group_state!(group, override {
        title, draw, handle, set_bounds, cursor, select, unselect
    });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn set_bounds(&mut self, r: Rect) {
        if self.group.bounds() != r {
            self.cancel_edit();
            self.filtering = false;
            self.sort_path_target = None;
        }
        self.group.set_bounds(r);
        self.group.set_child_bounds(0, Rect::new(0, 0, r.w(), r.h()));
        self.update_col_widths();
        self.layout_input_line();
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.inner_mut().state_mut().set_focused(true);
        self.group.mark_dirty();
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.inner_mut().state_mut().set_focused(false);
        self.group.mark_dirty();
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        if self.is_editing() {
            return self.group.cursor();
        }
        None
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }

    fn draw(&mut self) {
        // TreeTableView (child 0) renders itself via the group pipeline.
        // InputLine (child 1) bounds are set when editing starts and on set_bounds.
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command {
            id, broadcast: true, ..
        } = event
        {
            if *id == CM_FS_CHANGED {
                return handle::handle_save_command(self);
            }
            return HandleResult::Ignored;
        }
        if let Event::Command { .. } = event {
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if self.is_editing() {
            self.group.dispatch(event);
            handle::drain_edit_commands(self);
            self.group.mark_dirty();
            return HandleResult::Consumed;
        }
        handle::handle_struct_key(self, key)
    }
}
