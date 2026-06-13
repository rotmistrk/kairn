//! A single node in the JSON document arena.

use crate::structured::{NodeId, NodeKind, ScalarType};

/// A single node in the JSON document arena.
#[derive(Debug, Clone)]
pub struct Node {
    pub(crate) kind: NodeKind,
    pub(crate) key: Option<String>,
    pub(crate) value: String,
    pub(crate) meta: String,
    pub(crate) children: Vec<NodeId>,
    pub(crate) parent: Option<NodeId>,
    pub(crate) expanded: bool,
    pub(crate) inline: bool,
    pub(crate) scalar_type: ScalarType,
}

impl Node {
    pub(crate) fn kind(&self) -> NodeKind {
        self.kind
    }

    pub(crate) fn key(&self) -> Option<&str> {
        self.key.as_deref()
    }

    pub(crate) fn meta(&self) -> &str {
        &self.meta
    }

    pub(crate) fn children(&self) -> &[NodeId] {
        &self.children
    }

    pub(crate) fn parent(&self) -> Option<NodeId> {
        self.parent
    }

    pub(crate) fn inline(&self) -> bool {
        self.inline
    }

    pub(crate) fn scalar_type(&self) -> ScalarType {
        self.scalar_type
    }

    pub(crate) fn set_parent(&mut self, parent: Option<NodeId>) {
        self.parent = parent;
    }
}
