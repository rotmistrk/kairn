//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use txv_core::prelude::*;
use txv_widgets::inline_edit::{InlineEditResult, InlineEditor};
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use self::handle::HandleAction;

pub mod data;
mod handle;
mod mcp;
pub mod model;

pub use self::data::TodoTreeData;

/// The todo tree view — wraps TreeView<TodoTreeData>.
pub struct TodoTreeView {
    inner: TreeView<TodoTreeData>,
    editing: Option<InlineEditor>,
    confirm_delete: bool,
}

impl TodoTreeView {
    pub fn new(root: &Path) -> Self {
        let todo_path = root.join(".kairn.todo");
        let data = TodoTreeData::new(&todo_path);
        Self {
            inner: TreeView::new(data),
            editing: None,
            confirm_delete: false,
        }
    }

    fn start_edit(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new(row, &label));
            self.inner.state.mark_dirty();
        }
    }

    fn start_edit_selected(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            self.editing = Some(InlineEditor::new_selected(row, &label));
            self.inner.state.mark_dirty();
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
        self.inner.state.mark_dirty();
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
            HandleAction::EditNew(row) => {
                self.inner.cursor = row;
                self.start_edit_selected();
            }
            HandleAction::ConfirmDelete => {
                self.confirm_delete = true;
            }
        }
        self.inner.state.mark_dirty();
    }
}

impl View for TodoTreeView {
    delegate_view!(inner, override { title, handle, draw, can_close });

    fn title(&self) -> &str {
        "Todo"
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Denied("permanent tab".to_string())
    }

    fn draw(&self, surface: &mut Surface) {
        if self.inner.data.visible_count() == 0 {
            let b = self.inner.state.bounds();
            let dim = txv_core::palette::palette().base.dim.to_style();
            surface.print(b.x, b.y, "  (empty \u{2014} press 'n' to add)", dim);
            return;
        }
        // Custom draw with checkboxes
        let pal = txv_core::palette::palette();
        let b = self.inner.state.bounds();
        for row in 0..b.h as usize {
            let idx = self.inner.scroll.offset + row;
            if idx >= self.inner.data.visible_count() {
                break;
            }
            let id = self.inner.data.visible_id(idx);
            let depth = self.inner.data.depth(id);
            let indent = (depth * 2) as u16;
            let marker = if self.inner.data.is_expandable(id) {
                if self.inner.data.is_expanded(id) {
                    "▼ "
                } else {
                    "▶ "
                }
            } else {
                "  "
            };
            let node_style = self.inner.data.style(id);
            let style = if idx == self.inner.cursor {
                if self.inner.state.is_focused() {
                    pal.interactive.cursor_focused.resolve(&node_style)
                } else {
                    pal.interactive.cursor_unfocused.resolve(&node_style)
                }
            } else {
                node_style
            };
            let y = b.y + row as u16;
            surface.hline(b.x, y, b.w, ' ', style);
            let x = b.x + indent;
            surface.print(x, y, marker, style);
            // Checkbox
            let checkbox = if let Some(item) = self.inner.data.item_at(id) {
                match item.completed {
                    model::Completion::Done => "[x] ",
                    _ => "[ ] ",
                }
            } else {
                "[ ] "
            };
            surface.print(x + 2, y, checkbox, style);
            surface.print(x + 6, y, self.inner.data.label(id), style);
        }
        // Render inline editor overlay
        if let Some(ref editor) = self.editing {
            let b = self.inner.state.bounds();
            let scroll_offset = self.inner.scroll.offset;
            if editor.row >= scroll_offset {
                let screen_row = (editor.row - scroll_offset) as u16;
                if screen_row < b.h {
                    let y = b.y + screen_row;
                    let id = self.inner.data.visible_id(editor.row);
                    let depth = self.inner.data.depth(id);
                    let indent = (depth * 2 + 6) as u16; // marker(2) + checkbox(4)
                    let ex = b.x + indent;
                    let ew = b.w.saturating_sub(indent);
                    let style = pal.base.text.to_style();
                    editor.draw(surface, ex, y, ew, style);
                }
            }
        }
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return self.inner.handle(event, queue);
        };
        // Handle delete confirmation
        if self.confirm_delete {
            self.confirm_delete = false;
            if key.code == KeyCode::Char('y') {
                let cursor = self.inner.cursor;
                if cursor < self.inner.data.visible_count() {
                    let id = self.inner.data.visible_id(cursor);
                    if let Some(path) = self.inner.data.path_at(id) {
                        let path = path.clone();
                        model::remove_item(&mut self.inner.data.file, &path);
                        self.inner.data.save();
                        self.inner.data.rebuild_flat();
                    }
                }
            }
            let msg = txv_core::message::Message::info("todo", String::new());
            queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
            self.inner.state.mark_dirty();
            return HandleResult::Consumed;
        }
        if self.editing.is_some() {
            return self.handle_editing_key(key);
        }
        // 'n' works even on empty tree — adds first item
        if key.code == KeyCode::Char('n') && self.inner.data.visible_count() == 0 {
            self.inner.data.add_first_item();
            return HandleResult::Consumed;
        }
        if key.code == KeyCode::Char('e') && self.inner.data.visible_count() > 0 {
            self.start_edit();
            return HandleResult::Consumed;
        }
        let cursor = self.inner.cursor;
        if self.inner.data.visible_count() > 0 {
            if let Some(action) = handle::handle_todo_key(key, &mut self.inner.data, cursor, queue) {
                if matches!(action, HandleAction::ConfirmDelete) {
                    let msg = txv_core::message::Message::info("todo", "Delete item? [y]es [Esc]cancel".to_string());
                    queue.put_command(txv_widgets::CM_STATUS_MESSAGE, Some(Box::new(msg)));
                }
                self.apply_action(action);
                return HandleResult::Consumed;
            }
        }
        self.inner.handle(event, queue)
    }
}
