//! StructuredView — tree-table view for structured data (JSON, YAML, etc.).
//!
//! Uses TreeTableView<StructDocSource> as group child 0, with InputLine child 1 when editing.

mod edit;
mod handle;
pub(crate) mod source;
pub(crate) mod undo;
mod view_impl;

use std::fs;
use std::path::{Path, PathBuf};

use txv_core::prelude::*;
use txv_widgets::TreeTableView;

use crate::structured::{NodeId, StructuredDoc};

use source::StructDocSource;
use undo::UndoStack;

/// Which column currently has focus (0=key/tree, 1=value, 2=meta).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColFocus {
    Key,
    Value,
    Meta,
}

impl ColFocus {
    pub(crate) fn as_col_index(self) -> usize {
        match self {
            ColFocus::Key => 0,
            ColFocus::Value => 1,
            ColFocus::Meta => 2,
        }
    }
}

/// What the inline editor is targeting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    Value,
    Key,
    Meta,
}

/// Tree-table view for structured documents.
pub struct StructuredView {
    pub(crate) group: GroupState,
    pub(crate) path: PathBuf,
    pub(crate) col_focus: ColFocus,
    pub(crate) display_title: String,
    pub(crate) edit_target: EditTarget,
    pub(crate) editing_row: Option<usize>,
    pub(crate) dirty: bool,
    pub(crate) undo_stack: UndoStack,
    pub(crate) filtering: bool,
    pub(crate) last_sort_node: Option<NodeId>,
    pub(crate) last_sort_asc: bool,
    pub(crate) sort_path_target: Option<NodeId>,
    pub(crate) child_sink: EventSink,
    /// Yanked node JSON (for y/p).
    pub(crate) yanked: Option<String>,
}

impl StructuredView {
    pub fn new(path: &Path, doc: Box<dyn StructuredDoc>) -> Self {
        let display_title = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "structured".to_string());
        let source = StructDocSource::new(doc);
        let mut tree = TreeTableView::new(source, &[1, 1]);
        tree.set_focused_col(Some(0));
        let mut group = GroupState::default();
        group.insert(Box::new(tree));
        Self {
            group,
            path: path.to_path_buf(),
            col_focus: ColFocus::Key,
            display_title,
            edit_target: EditTarget::Value,
            editing_row: None,
            dirty: false,
            undo_stack: UndoStack::new(),
            filtering: false,
            last_sort_node: None,
            last_sort_asc: true,
            sort_path_target: None,
            child_sink: EventSink::new(),
            yanked: None,
        }
    }

    /// Typed access to the TreeTableView (always child 0).
    pub(crate) fn inner(&self) -> &TreeTableView<StructDocSource> {
        self.group
            .child(0)
            .and_then(|c| c.as_any())
            .and_then(|a| a.downcast_ref())
            .unwrap_or_else(|| unreachable!())
    }

    /// Typed mutable access to the TreeTableView (always child 0).
    pub(crate) fn inner_mut(&mut self) -> &mut TreeTableView<StructDocSource> {
        self.group
            .child_mut(0)
            .and_then(|c| c.as_any_mut())
            .and_then(|a| a.downcast_mut())
            .unwrap_or_else(|| unreachable!())
    }

    pub fn save(&mut self) -> Result<(), String> {
        let content = self.inner_mut().data_mut().doc().serialize();
        fs::write(&self.path, &content).map_err(|e| e.to_string())?;
        self.dirty = false;
        self.sync_title();
        Ok(())
    }

    pub(crate) fn rebuild_visible(&mut self) {
        self.inner_mut().data_mut().rebuild_visible();
    }

    pub(crate) fn sync_scroll(&mut self) {
        let max = self.inner_mut().data_mut().visible_nodes().len().saturating_sub(1);
        let cur = self.inner().cursor();
        if cur > max {
            self.inner_mut().set_cursor(max);
        }
    }

    pub(crate) fn clamp_cursor(&mut self) {
        let max = self.inner_mut().data_mut().visible_nodes().len().saturating_sub(1);
        if self.inner().cursor() > max {
            self.inner_mut().set_cursor(max);
        }
    }

    pub(crate) fn sync_title(&mut self) {
        let name = self
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "structured".to_string());
        let filter_mark = if self.inner_mut().data_mut().filter_text().is_empty() {
            String::new()
        } else {
            format!(" [filter: {}]", self.inner_mut().data_mut().filter_text())
        };
        self.display_title = format!("{name}{filter_mark}");
    }

    pub(crate) fn save_undo_point(&mut self) {
        let snapshot = self.inner_mut().data_mut().doc().snapshot();
        self.undo_stack.save_state(&snapshot);
    }

    pub(crate) fn apply_undo(&mut self) {
        let current = self.inner_mut().data_mut().doc().snapshot();
        self.undo_stack.bookmark_current(&current);
        let Some(snapshot) = self.undo_stack.undo().map(|s| s.to_string()) else {
            return;
        };
        if self.inner_mut().data_mut().doc_mut().restore(&snapshot).is_ok() {
            self.rebuild_visible();
            self.clamp_cursor();
            self.group.mark_dirty();
        }
    }

    pub(crate) fn apply_redo(&mut self) {
        let Some(snapshot) = self.undo_stack.redo().map(|s| s.to_string()) else {
            return;
        };
        if self.inner_mut().data_mut().doc_mut().restore(&snapshot).is_ok() {
            self.rebuild_visible();
            self.clamp_cursor();
            self.group.mark_dirty();
        }
    }

    /// Recompute column widths based on current bounds.
    pub(crate) fn update_col_widths(&mut self) {
        let w = self.group.bounds().w() as usize;
        let val_w = w * 40 / 100;
        let meta_w = w.saturating_sub(w * 40 / 100 + val_w + 2);
        self.inner_mut().set_col_widths(&[val_w as u16, meta_w as u16]);
    }

    /// Position the InputLine child at the editing cell location.
    pub(crate) fn layout_input_line(&mut self) {
        if self.group.child_count() < 2 {
            return;
        }
        let Some(row) = self.editing_row else {
            return;
        };
        let offset = self.inner().scroll_offset();
        let h = self.group.bounds().h() as usize;
        if row < offset || row >= offset + h {
            return;
        }
        let y = (row - offset) as u16;
        let col_idx = self.col_focus.as_col_index();
        let (col_x, col_w) = self.inner().column_bounds(col_idx);
        self.group.set_child_bounds(1, Rect::new(col_x, y, col_w, 1));
    }
}
