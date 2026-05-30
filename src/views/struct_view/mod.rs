//! StructuredView — tree-table view for structured data (JSON, YAML, etc.).

mod draw;
mod edit;
mod filter;
mod handle;
pub(crate) mod undo;

use std::fs;
use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::commands::CM_SAVE;
use crate::structured::{NodeId, NodeKind, StructuredDoc};

use undo::UndoStack;

/// Which column currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColFocus {
    Key,
    Value,
    Meta,
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
    pub(crate) doc: Box<dyn StructuredDoc>,
    pub(crate) path: PathBuf,
    pub(crate) cursor: usize,
    pub(crate) scroll: usize,
    pub(crate) col_focus: ColFocus,
    pub(crate) visible_nodes: Vec<NodeId>,
    pub(crate) display_title: String,
    pub(crate) edit_target: EditTarget,
    pub(crate) editing_row: Option<usize>,
    pub(crate) dirty: bool,
    pub(crate) undo_stack: UndoStack,
    pub(crate) filter_text: String,
    pub(crate) filtering: bool,
    pub(crate) last_sort_node: Option<NodeId>,
    pub(crate) last_sort_asc: bool,
    pub(crate) sort_path_target: Option<NodeId>,
    /// Sink for capturing InputLine commands.
    pub(crate) child_sink: EventSink,
}

impl StructuredView {
    pub fn new(path: &Path, doc: Box<dyn StructuredDoc>) -> Self {
        let display_title = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "structured".to_string());
        let mut view = Self {
            group: GroupState::default(),
            doc,
            path: path.to_path_buf(),
            cursor: 0,
            scroll: 0,
            col_focus: ColFocus::Key,
            visible_nodes: Vec::new(),
            display_title,
            edit_target: EditTarget::Value,
            editing_row: None,
            dirty: false,
            undo_stack: UndoStack::new(),
            filter_text: String::new(),
            filtering: false,
            last_sort_node: None,
            last_sort_asc: true,
            sort_path_target: None,
            child_sink: EventSink::new(),
        };
        view.rebuild_visible();
        view
    }

    pub(crate) fn rebuild_visible(&mut self) {
        self.visible_nodes.clear();
        let root = self.doc.root();
        if self.filter_text.is_empty() {
            self.dfs_collect(root);
        } else {
            self.dfs_collect_filtered(root);
        }
    }

    fn dfs_collect(&mut self, id: NodeId) {
        self.visible_nodes.push(id);
        if self.doc.node_kind(id) != NodeKind::Scalar && self.doc.is_expanded(id) {
            let children: Vec<NodeId> = self.doc.children(id).to_vec();
            for child in children {
                self.dfs_collect(child);
            }
        }
    }

    pub fn save(&mut self) -> Result<(), String> {
        let content = self.doc.serialize();
        fs::write(&self.path, &content).map_err(|e| e.to_string())?;
        self.dirty = false;
        self.sync_title();
        Ok(())
    }

    pub(crate) fn depth(&self, id: NodeId) -> usize {
        let mut d = 0;
        let mut current = id;
        while let Some(p) = self.doc.parent(current) {
            d += 1;
            current = p;
        }
        d
    }

    pub(crate) fn is_last_child(&self, id: NodeId) -> bool {
        if let Some(parent) = self.doc.parent(id) {
            let siblings = self.doc.children(parent);
            siblings.last() == Some(&id)
        } else {
            true
        }
    }

    pub(crate) fn sync_scroll(&mut self) {
        let h = self.group.bounds().h as usize;
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
    }

    pub(crate) fn clamp_cursor(&mut self) {
        if self.cursor >= self.visible_nodes.len() {
            self.cursor = self.visible_nodes.len().saturating_sub(1);
        }
    }

    pub(crate) fn sync_title(&mut self) {
        let name = self
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "structured".to_string());
        let filter_mark = if self.filter_text.is_empty() {
            String::new()
        } else {
            format!(" [filter: {}]", self.filter_text)
        };
        self.display_title = format!("{name}{filter_mark}");
    }

    pub(crate) fn save_undo_point(&mut self) {
        let snapshot = self.doc.snapshot();
        self.undo_stack.save_state(&snapshot);
    }

    pub(crate) fn apply_undo(&mut self) {
        let current = self.doc.snapshot();
        self.undo_stack.bookmark_current(&current);
        let Some(snapshot) = self.undo_stack.undo().map(|s| s.to_string()) else {
            return;
        };
        if self.doc.restore(&snapshot).is_ok() {
            self.rebuild_visible();
            self.clamp_cursor();
            self.sync_scroll();
            self.group.mark_dirty();
        }
    }

    pub(crate) fn apply_redo(&mut self) {
        let Some(snapshot) = self.undo_stack.redo().map(|s| s.to_string()) else {
            return;
        };
        if self.doc.restore(&snapshot).is_ok() {
            self.rebuild_visible();
            self.clamp_cursor();
            self.sync_scroll();
            self.group.mark_dirty();
        }
    }
}

impl View for StructuredView {
    delegate_group_state!(group, override { title, draw, handle, set_bounds, cursor, select, unselect });

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
        if self.is_editing() {
            return self.group.cursor();
        }
        None
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }

    fn draw(&mut self) {
        draw::draw_struct_view(self);
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
        // When editing, group dispatch routes to focused InputLine child
        if self.is_editing() {
            let _result = self.group.dispatch(event);
            handle::drain_edit_commands(self);
            self.group.mark_dirty();
            return HandleResult::Consumed;
        }
        handle::handle_struct_key(self, key)
    }
}
