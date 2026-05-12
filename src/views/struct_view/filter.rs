//! Filter logic for StructuredView — filtered DFS traversal.

use crate::structured::{NodeId, NodeKind};

use super::StructuredView;

impl StructuredView {
    pub(crate) fn dfs_collect_filtered(&mut self, id: NodeId) {
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

    /// Returns true if this node or any descendant matches the filter.
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
}
