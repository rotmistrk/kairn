//! Structured document model — format-agnostic tree representation.

pub mod json_doc;

/// Unique identifier for a node in the document arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(pub usize);

/// The structural kind of a node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Dict,
    Array,
    Scalar,
}

/// The type of a scalar value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarType {
    Null,
    Bool,
    Number,
    String,
}

/// Format-agnostic trait for navigating and mutating a structured document.
pub trait StructuredDoc: Send {
    fn root(&self) -> NodeId;
    fn children(&self, id: NodeId) -> &[NodeId];
    fn node_kind(&self, id: NodeId) -> NodeKind;
    fn key(&self, id: NodeId) -> Option<&str>;
    fn value_display(&self, id: NodeId) -> &str;
    fn meta(&self, id: NodeId) -> &str;
    fn is_inline(&self, id: NodeId) -> bool;
    fn is_expanded(&self, id: NodeId) -> bool;
    fn toggle_expand(&mut self, id: NodeId);
    fn set_key(&mut self, id: NodeId, key: &str) -> Result<(), String>;
    fn set_value(&mut self, id: NodeId, val: &str) -> Result<(), String>;
    fn set_meta(&mut self, id: NodeId, meta: &str);
    fn toggle_inline(&mut self, id: NodeId);
    fn add_sibling(&mut self, id: NodeId) -> Result<NodeId, String>;
    fn add_child(&mut self, id: NodeId) -> Result<NodeId, String>;
    fn clone_node(&mut self, id: NodeId) -> Result<NodeId, String>;
    fn remove(&mut self, id: NodeId) -> Result<(), String>;
    fn swap_up(&mut self, id: NodeId) -> Result<(), String>;
    fn swap_down(&mut self, id: NodeId) -> Result<(), String>;
    fn promote(&mut self, id: NodeId) -> Result<(), String>;
    fn demote(&mut self, id: NodeId) -> Result<(), String>;
    fn cycle_type(&mut self, id: NodeId);
    fn convert_container(&mut self, id: NodeId);
    fn serialize(&self) -> String;
    fn parent(&self, id: NodeId) -> Option<NodeId>;
    fn scalar_type(&self, id: NodeId) -> ScalarType;
    /// Snapshot the document state as a string (for undo).
    fn snapshot(&self) -> String;
    /// Restore document state from a snapshot string.
    fn restore(&mut self, snapshot: &str) -> Result<(), String>;
}
