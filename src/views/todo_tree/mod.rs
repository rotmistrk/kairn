//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::{TreeTableView, CM_ACTIVATE_GROUP, CM_DEACTIVATE_GROUP};

mod apply_action;
pub mod data;
mod dispatch;
mod draw;
mod edit;
mod flat_node;
mod handle;
mod mcp;
pub mod model;
mod ops;
mod source;

pub use self::data::TodoTreeData;

/// Group ID for the todo status bar section.
pub const TODO_STATUS_GROUP: u16 = 1;

/// The todo tree view — a Group that hosts TreeTableView (child 0) and InputLine (child 1 when editing).
pub struct TodoTreeView {
    group: GroupState,
    /// Sink for capturing InputLine commands (separate from group sink).
    child_sink: EventSink,
    /// Editing state: which visible row is being edited.
    editing_row: Option<usize>,
    /// Filter mode active (InputLine child is the filter).
    filter_active: bool,
    /// Pending crypto path for passphrase prompt.
    crypto_pending: Option<CryptoPending>,
    /// Shared clipboard ring.
    pub(crate) clipboard: Option<txv_core::clipboard_ring::ClipboardHandle>,
}

enum CryptoPending {
    Encrypt(model::TreePath),
    Decrypt(model::TreePath),
}

impl TodoTreeView {
    pub fn new(root: &Path) -> Self {
        let todo_path = root.join(".kairn.todo");
        let data = TodoTreeData::new(&todo_path);
        let mut group = GroupState::default();
        group.insert(Box::new(TreeTableView::new(data, &[3, 2])));
        Self {
            group,
            child_sink: EventSink::new(),
            editing_row: None,
            filter_active: false,
            crypto_pending: None,
            clipboard: None,
        }
    }

    /// Typed access to the TreeTableView (always child 0).
    pub(crate) fn inner(&self) -> &TreeTableView<TodoTreeData> {
        self.group
            .child(0)
            .and_then(|c| c.as_any())
            .and_then(|a| a.downcast_ref())
            .unwrap_or_else(|| unreachable!())
    }

    /// Typed mutable access to the TreeTableView (always child 0).
    pub(crate) fn inner_mut(&mut self) -> &mut TreeTableView<TodoTreeData> {
        self.group
            .child_mut(0)
            .and_then(|c| c.as_any_mut())
            .and_then(|a| a.downcast_mut())
            .unwrap_or_else(|| unreachable!())
    }
}

impl View for TodoTreeView {
    delegate_group_state!(group, override { title, handle, draw, cursor, select, unselect, set_bounds });

    fn title(&self) -> &str {
        "Todo"
    }

    fn set_bounds(&mut self, r: Rect) {
        if self.group.bounds() != r {
            self.commit_edit_on_resize();
        }
        self.group.set_bounds(r);
        // Position TreeTableView (child 0) — fills area minus optional filter row
        let h = r.h();
        let w = r.w();
        let has_filter = self.filter_active || !self.inner_mut().data_mut().filter_text().is_empty();
        let draw_h = if has_filter {
            h.saturating_sub(1)
        } else {
            h
        };
        self.group.set_child_bounds(0, Rect::new(0, 0, w, draw_h));
        self.sync_tree_visibility();
        self.layout_edit_child();
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.group.mark_dirty();
        self.group
            .put_command(CM_ACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.group.mark_dirty();
        self.group
            .put_command(CM_DEACTIVATE_GROUP, Some(Box::new(TODO_STATUS_GROUP)));
    }

    fn cursor(&self) -> Option<txv_core::cursor::CursorRequest> {
        // When editing, cursor comes from the InputLine child
        if self.is_editing() {
            return self.group.cursor();
        }
        None
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn draw(&mut self) {
        self.draw_tree();
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if matches!(event, Event::Tick) {
            if self.inner_mut().data_mut().reload_if_changed() {
                self.sync_tree_visibility();
            }
            return HandleResult::Ignored;
        }
        if let Event::Command { id, .. } = event {
            if self.editing_row.is_some() {
                self.group.dispatch(event);
                self.drain_edit_commands();
                self.group.mark_dirty();
                return HandleResult::Consumed;
            }
            return self.handle_status_command(*id);
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if self.filter_active {
            return self.handle_filter_key(key, event);
        }
        if self.editing_row.is_some() {
            if self.handle_edit_paste(event) {
                return HandleResult::Consumed;
            }
            let _result = self.group.dispatch(event);
            self.drain_edit_commands();
            self.group.mark_dirty();
            return HandleResult::Consumed;
        }
        self.handle_normal_key(key, event)
    }
}
