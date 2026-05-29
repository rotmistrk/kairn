//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;

use duir_core::crypto::{decrypt_item, encrypt_item};
use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

use crate::commands::CM_TODO_NOTE_UPDATE;

mod apply_action;
pub mod data;
mod dispatch;
mod draw;
mod flat_node;
mod handle;
mod mcp;
pub mod model;

pub use self::data::TodoTreeData;

/// The todo tree view — wraps TreeView<TodoTreeData>.
pub struct TodoTreeView {
    inner: TreeView<TodoTreeData>,
    /// Active item editor (row being edited stored separately).
    editing: Option<(usize, InputLine)>,
    /// Active filter editor.
    filter_editor: Option<InputLine>,
    /// Sink to capture InputLine commands.
    edit_sink: EventSink,
    /// Pending crypto path for passphrase prompt.
    crypto_pending: Option<CryptoPending>,
}

enum CryptoPending {
    Encrypt(model::TreePath),
    Decrypt(model::TreePath),
}

impl TodoTreeView {
    pub fn new(root: &Path) -> Self {
        let todo_path = root.join(".kairn.todo");
        let data = TodoTreeData::new(&todo_path);
        Self {
            inner: TreeView::new(data),
            editing: None,
            filter_editor: None,
            edit_sink: EventSink::new(),
            crypto_pending: None,
        }
    }

    /// Emit CM_TODO_NOTE_UPDATE if cursor moved to a different item.
    fn emit_note_update_if_cursor_changed(&mut self, prev_cursor: usize) {
        if self.inner.cursor == prev_cursor {
            return;
        }
        let cursor = self.inner.cursor;
        if cursor >= self.inner.data.visible_count() {
            return;
        }
        let id = self.inner.data.visible_id(cursor);
        if let Some(path) = self.inner.data.path_at(id) {
            let path = path.clone();
            if let Some(item) = model::get_item(&self.inner.data.file, &path) {
                let note = item.note.clone();
                self.inner
                    .state
                    .put_command(CM_TODO_NOTE_UPDATE, Some(Box::new((path, note))));
            }
        }
    }

    /// Execute the pending delete (called from handler_confirm on 'y' response).
    pub fn confirm_delete_execute(&mut self) {
        let cursor = self.inner.cursor;
        if cursor < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(cursor);
            if let Some(path) = self.inner.data.path_at(id) {
                let path = path.clone();
                model::remove_item(&mut self.inner.data.file, &path);
                model::propagate_completion(&mut self.inner.data.file, &path);
                self.inner.data.save();
                self.inner.data.rebuild_flat();
                self.inner.mark_dirty();
            }
        }
    }

    /// Execute crypto passphrase response (called from handler_confirm on 'y'/commit).
    pub fn crypto_passphrase_response(&mut self, passphrase: &str) {
        let Some(pending) = self.crypto_pending.take() else {
            return;
        };
        match pending {
            CryptoPending::Encrypt(path) => {
                if let Some(item) = model::get_item_mut(&mut self.inner.data.file, &path) {
                    if let Err(e) = encrypt_item(item, passphrase) {
                        log::warn!("encrypt failed: {e}");
                    }
                }
            }
            CryptoPending::Decrypt(path) => {
                if let Some(item) = model::get_item_mut(&mut self.inner.data.file, &path) {
                    if let Err(e) = decrypt_item(item, passphrase) {
                        log::warn!("decrypt failed: {e}");
                    }
                }
            }
        }
        self.inner.data.save();
        self.inner.data.rebuild_flat();
        self.inner.mark_dirty();
    }

    fn start_edit(&mut self) {
        let row = self.inner.cursor;
        if row < self.inner.data.visible_count() {
            let id = self.inner.data.visible_id(row);
            let label = self.inner.data.label(id).to_owned();
            let mut input = InputLine::new().with_command(CM_OK).with_inherit_bg();
            input.set_sink(self.edit_sink.clone());
            input.set_text(&label);
            input.select_all();
            self.editing = Some((row, input));
            self.inner.mark_dirty();
        }
    }

    fn start_edit_selected(&mut self) {
        self.start_edit();
    }

    fn handle_editing_key(&mut self, key: &KeyEvent) -> HandleResult {
        let Some((_, ref mut input)) = self.editing else {
            return HandleResult::Ignored;
        };
        input.handle(&Event::Key(*key));
        // Check for commands from InputLine
        for ev in self.edit_sink.drain() {
            if let Event::Command { id, data, .. } = ev {
                match id {
                    CM_OK => {
                        let text = data
                            .and_then(|d| d.downcast::<String>().ok())
                            .map(|s| *s)
                            .unwrap_or_default();
                        let row = self.editing.as_ref().map(|(r, _)| *r).unwrap_or(0);
                        self.editing = None;
                        self.inner.data.update_title(row, text);
                        self.inner.mark_dirty();
                        return HandleResult::Consumed;
                    }
                    CM_CANCEL => {
                        self.editing = None;
                        self.inner.mark_dirty();
                        return HandleResult::Consumed;
                    }
                    _ => {}
                }
            }
        }
        self.inner.mark_dirty();
        HandleResult::Consumed
    }
}

impl View for TodoTreeView {
    delegate_view!(inner, override { title, handle, draw, can_close, set_bounds });

    fn title(&self) -> &str {
        "Todo"
    }

    fn set_bounds(&mut self, r: txv_core::geometry::Rect) {
        if self.inner.bounds() != r {
            self.commit_edit_on_resize();
        }
        self.inner.set_bounds(r);
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
            if self.inner.data.reload_if_changed() {
                self.inner.mark_dirty();
            }
            return self.inner.handle(event);
        }
        let Event::Key(key) = event else {
            return self.inner.handle(event);
        };
        if self.filter_editor.is_some() {
            return self.handle_filter_key(key);
        }
        if self.editing.is_some() {
            return self.handle_editing_key(key);
        }
        self.handle_normal_key(key, event)
    }
}

impl TodoTreeView {
    /// Commit any active inline edit when the view is resized.
    fn commit_edit_on_resize(&mut self) {
        if let Some((row, ref input)) = self.editing.take() {
            self.inner.data.update_title(row, input.text().to_string());
        }
        if self.filter_editor.is_some() {
            self.filter_editor = None;
        }
    }
}
