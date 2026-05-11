//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::inline_edit::{InlineEditResult, InlineEditor};
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
    editing: Option<InlineEditor>,
}

impl TodoTreeView {
    pub fn new(root: &Path) -> Self {
        let todo_path = root.join(".kairn.todo");
        let data = TodoTreeData::new(&todo_path);
        Self {
            inner: TreeView::new(data),
            editing: None,
        }
    }

    fn start_edit(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new(row, &label));
            self.inner.state.dirty = true;
        }
    }

    fn handle_editing_key(&mut self, key: &KeyEvent) -> HandleResult {
        let Some(ref mut editor) = self.editing else {
            return HandleResult::Ignored;
        };
        match editor.handle_key(key) {
            InlineEditResult::Continue => {}
            InlineEditResult::Commit(text) => {
                let row = editor.row;
                self.editing = None;
                self.inner.data.update_title(row, text);
            }
            InlineEditResult::Cancel => {
                self.editing = None;
            }
        }
        self.inner.state.dirty = true;
        HandleResult::Consumed
    }

    fn apply_action(&mut self, action: HandleAction) {
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
    }
}

impl View for TodoTreeView {
    delegate_view!(inner, override { title, handle, draw, can_close });

    fn title(&self) -> &str {
        "Todo"
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn draw(&self, surface: &mut Surface) {
        self.inner.draw(surface);
        if let Some(ref editor) = self.editing {
            let b = self.inner.state.bounds;
            let scroll_offset = self.inner.scroll.offset;
            if editor.row >= scroll_offset {
                let screen_row = (editor.row - scroll_offset) as u16;
                if screen_row < b.h {
                    let y = b.y + screen_row;
                    let style = Style {
                        fg: Color::Ansi(0),
                        bg: Color::Ansi(3),
                        ..Style::default()
                    };
                    editor.draw(surface, b.x, y, b.w, style);
                }
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return self.inner.handle(event, queue);
        };
        if self.editing.is_some() {
            return self.handle_editing_key(key);
        }
        if key.code == KeyCode::Char('e') && self.inner.data.visible_count() > 0 {
            self.start_edit();
            return HandleResult::Consumed;
        }
        let cursor = self.inner.cursor;
        if self.inner.data.visible_count() > 0 {
            if let Some(action) = handle::handle_todo_key(key, &mut self.inner.data, cursor, queue) {
                self.apply_action(action);
                return HandleResult::Consumed;
            }
        }
        self.inner.handle(event, queue)
    }
}
