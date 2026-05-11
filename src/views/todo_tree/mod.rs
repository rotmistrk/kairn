//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use self::handle::HandleAction;

pub mod data;
mod handle;
pub mod model;

pub use self::data::TodoTreeData;

/// The todo tree view — wraps TreeView<TodoTreeData>.
pub struct TodoTreeView {
    inner: TreeView<TodoTreeData>,
}

impl TodoTreeView {
    pub fn new(root: &Path) -> Self {
        let todo_path = root.join(".kairn.todo");
        let data = TodoTreeData::new(&todo_path);
        Self {
            inner: TreeView::new(data),
        }
    }
}

impl View for TodoTreeView {
    delegate_view!(inner, override { title, handle, can_close });

    fn title(&self) -> &str {
        "Todo"
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        if let Event::Key(key) = event {
            let cursor = self.inner.cursor;
            if self.inner.data.visible_count() > 0 {
                if let Some(action) = handle::handle_todo_key(key, &mut self.inner.data, cursor, queue) {
                    match action {
                        HandleAction::Stay => {}
                        HandleAction::MoveDown => {
                            let max = self.inner.data.visible_count().saturating_sub(1);
                            if self.inner.cursor < max {
                                self.inner.cursor += 1;
                            }
                        }
                        HandleAction::MoveTo(row) => {
                            self.inner.cursor = row;
                        }
                    }
                    self.inner.state.dirty = true;
                    return HandleResult::Consumed;
                }
            }
        }
        self.inner.handle(event, queue)
    }
}
