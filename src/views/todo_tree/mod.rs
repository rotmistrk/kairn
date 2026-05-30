//! TodoTreeView — non-closeable tab showing hierarchical tasks from .kairn.todo.

use std::path::Path;
use std::sync::Arc;

use txv_core::prelude::*;
use txv_widgets::input_line::InputLine;
use txv_widgets::tree_view::TreeData;
use txv_widgets::TreeView;

mod apply_action;
pub mod data;
mod dispatch;
mod draw;
mod flat_node;
mod handle;
mod mcp;
pub mod model;
mod ops;

pub use self::data::TodoTreeData;

/// The todo tree view — a Group that hosts an InputLine child when editing.
pub struct TodoTreeView {
    group: GroupState,
    inner: TreeView<TodoTreeData>,
    /// Sink for capturing InputLine commands (separate from group sink).
    child_sink: EventSink,
    /// Editing state: which visible row is being edited.
    editing_row: Option<usize>,
    /// Filter mode active (InputLine child is the filter).
    filter_active: bool,
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
            group: GroupState::default(),
            inner: TreeView::new(data),
            child_sink: EventSink::new(),
            editing_row: None,
            filter_active: false,
            crypto_pending: None,
        }
    }

    /// Start editing the current item title.
    fn start_edit(&mut self) {
        let row = self.inner.cursor;
        if row >= self.inner.data.visible_count() {
            return;
        }
        let id = self.inner.data.visible_id(row);
        let label = self.inner.data.label(id).to_owned();
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(&label);
        input.select_all();
        let pal = self.edit_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        self.group.set_focused_index(0);
        if let Some(child) = self.group.child_mut(0) {
            child.set_sink(sink);
            child.set_palette(pal);
            child.select();
        }
        self.editing_row = Some(row);
        self.group.mark_dirty();
    }

    fn start_edit_selected(&mut self) {
        self.start_edit();
    }

    /// Start filter mode.
    fn start_filter(&mut self) {
        let mut input = InputLine::new().with_command(CM_OK);
        input.set_text(&self.inner.data.filter_text.clone());
        let pal = self.filter_palette();
        let sink = self.child_sink.clone();
        self.group.insert(Box::new(input));
        self.group.set_focused_index(0);
        if let Some(child) = self.group.child_mut(0) {
            child.set_sink(sink);
            child.set_palette(pal);
            child.select();
        }
        self.filter_active = true;
        self.group.mark_dirty();
    }

    /// Palette for item editing: Text = cursor row style (focused).
    fn edit_palette(&self) -> Arc<dyn Palette> {
        use txv_core::palette::{palette, DerivedPalette, StyleId};
        let base = palette();
        let cursor_style = base.style(StyleId::CursorFocused);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, cursor_style))
    }

    /// Palette for filter bar: Text = StatusBar style.
    fn filter_palette(&self) -> Arc<dyn Palette> {
        use txv_core::palette::{palette, DerivedPalette, StyleId};
        let base = palette();
        let sb_style = base.style(StyleId::StatusBar);
        Arc::new(DerivedPalette::new(base).with_override(StyleId::Text, sb_style))
    }

    /// Get the InputLine child mutably.
    fn input_line_mut(&mut self) -> Option<&mut InputLine> {
        if self.group.child_count() > 0 {
            self.group
                .child_mut(0)
                .and_then(|c| c.as_any_mut()?.downcast_mut::<InputLine>())
        } else {
            None
        }
    }

    /// Remove the InputLine child.
    fn remove_input_line(&mut self) {
        if self.group.child_count() > 0 {
            self.group.remove(0);
        }
    }

    /// Commit the active edit.
    fn commit_edit(&mut self) {
        let text = self.input_line_mut().map(|i| i.text().to_string()).unwrap_or_default();
        self.remove_input_line();
        if let Some(row) = self.editing_row.take() {
            self.inner.data.update_title(row, text);
        }
        self.group.mark_dirty();
    }

    /// Cancel the active edit.
    fn cancel_edit(&mut self) {
        self.remove_input_line();
        self.editing_row = None;
        self.group.mark_dirty();
    }

    /// Commit filter (keep filter text, remove InputLine).
    fn commit_filter(&mut self) {
        self.remove_input_line();
        self.filter_active = false;
        self.group.mark_dirty();
    }

    /// Cancel filter (clear filter text, remove InputLine).
    fn cancel_filter(&mut self) {
        self.remove_input_line();
        self.filter_active = false;
        self.inner.data.filter_text.clear();
        self.inner.data.rebuild_flat();
        self.inner.cursor = 0;
        self.group.mark_dirty();
    }

    /// Whether we're in any editing mode.
    fn is_editing(&self) -> bool {
        self.editing_row.is_some() || self.filter_active
    }

    /// Emit CM_TODO_NOTE_UPDATE if cursor moved to a different item.
    /// Commit any active edit on resize.
    fn commit_edit_on_resize(&mut self) {
        if self.editing_row.is_some() {
            self.commit_edit();
        } else if self.filter_active {
            self.commit_filter();
        }
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
    }

    fn select(&mut self) {
        self.group.set_focused(true);
        self.group.mark_dirty();
    }

    fn unselect(&mut self) {
        self.group.set_focused(false);
        self.group.mark_dirty();
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
            if self.inner.data.reload_if_changed() {
                self.group.mark_dirty();
            }
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        if self.filter_active {
            // Group dispatch routes to focused InputLine child
            let result = self.group.dispatch(event);
            self.drain_child_sink();
            if let Some(input) = self.input_line_mut() {
                self.inner.data.filter_text = input.text().to_string();
            }
            self.inner.data.rebuild_flat();
            self.inner.cursor = 0;
            self.group.mark_dirty();
            if key.code == KeyCode::Esc {
                self.cancel_filter();
            } else if key.code == KeyCode::Enter {
                self.commit_filter();
            }
            return if result == HandleResult::Consumed {
                result
            } else {
                HandleResult::Consumed
            };
        }
        if self.editing_row.is_some() {
            // Group dispatch routes to focused InputLine child
            let _result = self.group.dispatch(event);
            self.drain_edit_commands();
            self.group.mark_dirty();
            return HandleResult::Consumed;
        }
        self.handle_normal_key(key, event)
    }
}
