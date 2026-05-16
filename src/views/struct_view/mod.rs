//! StructuredView — tree-table view for structured data (JSON, YAML, etc.).

mod draw;
mod filter;
mod handle;
pub(crate) mod undo;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;
use txv_widgets::inline_edit::InlineEditor;

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
    pub(crate) state: ViewState,
    pub(crate) doc: Box<dyn StructuredDoc>,
    pub(crate) path: PathBuf,
    pub(crate) cursor: usize,
    pub(crate) scroll: usize,
    pub(crate) col_focus: ColFocus,
    pub(crate) visible_nodes: Vec<NodeId>,
    pub(crate) display_title: String,
    pub(crate) editing: Option<InlineEditor>,
    pub(crate) edit_target: EditTarget,
    pub(crate) dirty: bool,
    pub(crate) undo_stack: UndoStack,
    pub(crate) filter_text: String,
    pub(crate) filtering: bool,
    pub(crate) last_sort_node: Option<NodeId>,
    pub(crate) last_sort_asc: bool,
    pub(crate) sort_path_target: Option<NodeId>,
}

impl StructuredView {
    pub fn new(path: &Path, doc: Box<dyn StructuredDoc>) -> Self {
        let display_title = path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "structured".to_string());
        let mut view = Self {
            state: ViewState::default(),
            doc,
            path: path.to_path_buf(),
            cursor: 0,
            scroll: 0,
            col_focus: ColFocus::Key,
            visible_nodes: Vec::new(),
            display_title,
            editing: None,
            edit_target: EditTarget::Value,
            dirty: false,
            undo_stack: UndoStack::new(),
            filter_text: String::new(),
            filtering: false,
            last_sort_node: None,
            last_sort_asc: true,
            sort_path_target: None,
        };
        view.rebuild_visible();
        view
    }

    /// Rebuild the flattened visible node list via DFS, only including expanded nodes.
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

    /// Save the document to disk.
    pub fn save(&mut self) -> Result<(), String> {
        let content = self.doc.serialize();
        std::fs::write(&self.path, &content).map_err(|e| e.to_string())?;
        self.dirty = false;
        self.sync_title();
        Ok(())
    }

    /// Depth of a node (number of ancestors).
    pub(crate) fn depth(&self, id: NodeId) -> usize {
        let mut d = 0;
        let mut current = id;
        while let Some(p) = self.doc.parent(current) {
            d += 1;
            current = p;
        }
        d
    }

    /// Whether a node is the last child of its parent.
    pub(crate) fn is_last_child(&self, id: NodeId) -> bool {
        if let Some(parent) = self.doc.parent(id) {
            let siblings = self.doc.children(parent);
            siblings.last() == Some(&id)
        } else {
            true
        }
    }

    /// Ensure cursor stays within visible range after scroll.
    pub(crate) fn sync_scroll(&mut self) {
        let h = self.state.bounds().h as usize;
        if h == 0 {
            return;
        }
        if self.cursor < self.scroll {
            self.scroll = self.cursor;
        } else if self.cursor >= self.scroll + h {
            self.scroll = self.cursor - h + 1;
        }
    }

    /// Start inline editing for the current cursor position and column focus.
    pub(crate) fn start_edit(&mut self, target: EditTarget) {
        let Some(&node_id) = self.visible_nodes.get(self.cursor) else {
            return;
        };
        let text = match target {
            EditTarget::Value => self.doc.value_display(node_id).to_owned(),
            EditTarget::Key => self.doc.key(node_id).unwrap_or("").to_owned(),
            EditTarget::Meta => self.doc.meta(node_id).to_owned(),
        };
        self.edit_target = target;
        self.editing = Some(InlineEditor::new(self.cursor, &text));
        self.state.mark_dirty();
    }

    /// Commit the current inline edit.
    pub(crate) fn commit_edit(&mut self) -> Option<String> {
        let editor = self.editing.take()?;
        let &node_id = self.visible_nodes.get(editor.row)?;
        let text = editor.buffer;
        let result = match self.edit_target {
            EditTarget::Value => self.doc.set_value(node_id, &text),
            EditTarget::Key => self.doc.set_key(node_id, &text),
            EditTarget::Meta => {
                self.doc.set_meta(node_id, &text);
                Ok(())
            }
        };
        self.dirty = true;
        self.sync_title();
        self.state.mark_dirty();
        result.err()
    }

    /// Cancel the current inline edit.
    pub(crate) fn cancel_edit(&mut self) {
        self.editing = None;
        self.state.mark_dirty();
    }

    /// Clamp cursor to valid range after structural changes.
    pub(crate) fn clamp_cursor(&mut self) {
        if self.cursor >= self.visible_nodes.len() {
            self.cursor = self.visible_nodes.len().saturating_sub(1);
        }
    }

    /// Update display_title based on dirty state and filter.
    pub(crate) fn sync_title(&mut self) {
        let name = self
            .path
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "structured".to_string());
        let dirty_mark = if self.dirty {
            " *"
        } else {
            ""
        };
        let filter_mark = if self.filter_text.is_empty() {
            String::new()
        } else {
            format!(" [filter: {}]", self.filter_text)
        };
        self.display_title = format!("{name}{dirty_mark}{filter_mark}");
    }

    /// Save current document state as an undo point.
    pub(crate) fn save_undo_point(&mut self) {
        let snapshot = self.doc.snapshot();
        self.undo_stack.save_state(&snapshot);
    }

    /// Undo: restore previous document state.
    pub(crate) fn apply_undo(&mut self) {
        // Bookmark current state so redo can get back to it
        let current = self.doc.snapshot();
        self.undo_stack.bookmark_current(&current);
        let Some(snapshot) = self.undo_stack.undo().map(|s| s.to_string()) else {
            return;
        };
        if self.doc.restore(&snapshot).is_ok() {
            self.rebuild_visible();
            self.clamp_cursor();
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }

    /// Redo: restore next document state.
    pub(crate) fn apply_redo(&mut self) {
        let Some(snapshot) = self.undo_stack.redo().map(|s| s.to_string()) else {
            return;
        };
        if self.doc.restore(&snapshot).is_ok() {
            self.rebuild_visible();
            self.clamp_cursor();
            self.sync_scroll();
            self.state.mark_dirty();
        }
    }
}

impl View for StructuredView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn draw(&mut self) {
        draw::draw_struct_view(self);
    }

    fn handle(&mut self, event: &Event) -> HandleResult {
        if let Event::Command { id, .. } = event {
            if *id == crate::commands::CM_SAVE {
                return handle::handle_save_command(self);
            }
            return HandleResult::Ignored;
        }
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        handle::handle_struct_key(self, key)
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }
}
