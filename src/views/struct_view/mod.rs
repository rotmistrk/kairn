//! StructuredView — tree-table view for structured data (JSON, YAML, etc.).

mod draw;
mod handle;

use std::path::{Path, PathBuf};

use txv_core::prelude::*;

use crate::structured::{NodeId, NodeKind, StructuredDoc};

/// Which column currently has focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColFocus {
    Key,
    Value,
    Meta,
}

/// Tree-table view for structured documents.
pub struct StructuredView {
    state: ViewState,
    doc: Box<dyn StructuredDoc>,
    path: PathBuf,
    cursor: usize,
    scroll: usize,
    col_focus: ColFocus,
    visible_nodes: Vec<NodeId>,
    display_title: String,
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
        };
        view.rebuild_visible();
        view
    }

    /// Rebuild the flattened visible node list via DFS, only including expanded nodes.
    pub(crate) fn rebuild_visible(&mut self) {
        self.visible_nodes.clear();
        let root = self.doc.root();
        self.dfs_collect(root);
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
}

impl View for StructuredView {
    delegate_view_state!(state, override { title, draw, handle });

    fn title(&self) -> &str {
        &self.display_title
    }

    fn draw(&self, surface: &mut Surface) {
        draw::draw_struct_view(self, surface);
    }

    fn handle(&mut self, event: &Event, queue: &mut EventQueue) -> HandleResult {
        let Event::Key(key) = event else {
            return HandleResult::Ignored;
        };
        handle::handle_struct_key(self, key, queue)
    }

    fn can_close(&self) -> CloseResult {
        CloseResult::Ok
    }
}
