//! StructDocSource — TreeTableSource adapter for StructuredDoc.
//!
//! Wraps a StructuredDoc + visible_nodes with cached labels for TreeTableView.
//! Uses raw_labels mode: the label includes tree guides and expand markers.

use crate::structured::{NodeId, NodeKind, StructuredDoc};
use txv_widgets::TreeTableSource;

/// Adapter providing TreeTableSource over a StructuredDoc.
pub struct StructDocSource {
    pub(crate) doc: Box<dyn StructuredDoc>,
    pub(crate) visible_nodes: Vec<NodeId>,
    pub(crate) labels: Vec<String>,
    pub(crate) filter_text: String,
}

impl StructDocSource {
    pub fn new(doc: Box<dyn StructuredDoc>) -> Self {
        let mut s = Self {
            doc,
            visible_nodes: Vec::new(),
            labels: Vec::new(),
            filter_text: String::new(),
        };
        s.rebuild_visible();
        s
    }

    pub fn rebuild_visible(&mut self) {
        self.visible_nodes.clear();
        let root = self.doc.root();
        if self.filter_text.is_empty() {
            self.dfs_collect(root);
        } else {
            self.dfs_collect_filtered(root);
        }
        self.rebuild_labels();
    }

    fn rebuild_labels(&mut self) {
        self.labels.clear();
        self.labels.reserve(self.visible_nodes.len());
        for &node_id in &self.visible_nodes.clone() {
            self.labels.push(self.build_label(node_id));
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

    fn dfs_collect_filtered(&mut self, id: NodeId) {
        if !self.node_matches_filter_tree(id) {
            return;
        }
        self.visible_nodes.push(id);
        if self.doc.node_kind(id) != NodeKind::Scalar && self.doc.is_expanded(id) {
            let children: Vec<NodeId> = self.doc.children(id).to_vec();
            for child in children {
                self.dfs_collect_filtered(child);
            }
        }
    }

    fn node_matches_filter_tree(&self, id: NodeId) -> bool {
        let filter = self.filter_text.to_lowercase();
        if self.node_matches_filter(id, &filter) {
            return true;
        }
        if self.doc.node_kind(id) != NodeKind::Scalar {
            let children = self.doc.children(id).to_vec();
            for child in children {
                if self.node_matches_filter_tree(child) {
                    return true;
                }
            }
        }
        false
    }

    fn node_matches_filter(&self, id: NodeId, filter: &str) -> bool {
        if let Some(key) = self.doc.key(id) {
            if key.to_lowercase().contains(filter) {
                return true;
            }
        }
        self.doc.value_display(id).to_lowercase().contains(filter)
    }

    pub fn depth(&self, id: NodeId) -> usize {
        let mut d = 0;
        let mut current = id;
        while let Some(p) = self.doc.parent(current) {
            d += 1;
            current = p;
        }
        d
    }

    pub fn is_last_child(&self, id: NodeId) -> bool {
        if let Some(parent) = self.doc.parent(id) {
            let siblings = self.doc.children(parent);
            siblings.last() == Some(&id)
        } else {
            true
        }
    }

    fn build_label(&self, node_id: NodeId) -> String {
        let depth = self.depth(node_id);
        let mut text = self.build_tree_guides(node_id, depth);
        self.append_expand_marker(node_id, &mut text);
        self.append_key_label(node_id, depth, &mut text);
        text
    }

    fn build_tree_guides(&self, node_id: NodeId, depth: usize) -> String {
        let mut text = String::new();
        if depth > 0 {
            let mut guides = Vec::with_capacity(depth.saturating_sub(1));
            let mut current = node_id;
            for _ in 0..depth.saturating_sub(1) {
                if let Some(parent) = self.doc.parent(current) {
                    current = parent;
                    guides.push(!self.is_last_child(current));
                }
            }
            guides.reverse();
            for has_line in &guides {
                text.push_str(if *has_line {
                    "│ "
                } else {
                    "  "
                });
            }
            text.push_str(if self.is_last_child(node_id) {
                "└─"
            } else {
                "├─"
            });
        }
        text
    }

    fn append_expand_marker(&self, node_id: NodeId, text: &mut String) {
        if self.doc.node_kind(node_id) != NodeKind::Scalar {
            text.push(if self.doc.is_expanded(node_id) {
                '▼'
            } else {
                '▶'
            });
            text.push(' ');
        }
    }

    fn append_key_label(&self, node_id: NodeId, depth: usize, text: &mut String) {
        if let Some(key) = self.doc.key(node_id) {
            text.push_str(key);
        } else if depth > 0 {
            if let Some(parent) = self.doc.parent(node_id) {
                let siblings = self.doc.children(parent);
                if let Some(pos) = siblings.iter().position(|&c| c == node_id) {
                    text.push_str(&format!("[{pos}]"));
                }
            }
        }
    }
}

impl TreeTableSource for StructDocSource {
    fn visible_count(&self) -> usize {
        self.visible_nodes.len()
    }

    fn label(&self, row: usize) -> &str {
        self.labels.get(row).map(|s| s.as_str()).unwrap_or("")
    }

    fn depth(&self, _row: usize) -> usize {
        0
    }

    fn is_expandable(&self, _row: usize) -> bool {
        false
    }

    fn is_expanded(&self, _row: usize) -> bool {
        false
    }

    fn toggle(&mut self, _row: usize) {}

    fn column_count(&self) -> usize {
        2
    }

    fn cell(&self, row: usize, col: usize) -> &str {
        let Some(&node_id) = self.visible_nodes.get(row) else {
            return "";
        };
        match col {
            0 => self.doc.value_display(node_id),
            1 => self.doc.meta(node_id),
            _ => "",
        }
    }

    fn raw_labels(&self) -> bool {
        true
    }

    fn filter_status(&self) -> Option<&str> {
        if self.filter_text.is_empty() {
            None
        } else {
            Some(&self.filter_text)
        }
    }
}
