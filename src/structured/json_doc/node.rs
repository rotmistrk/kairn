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
