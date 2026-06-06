//! JSONL (one JSON value per line) implementation of StructuredDoc.
//! Wraps JsonDoc — presents lines as a synthetic root array.

use serde_json::Value;

use crate::structured::json_doc::JsonDoc;
use crate::structured::{NodeId, NodeKind, ScalarType, StructuredDoc};

/// JSONL document — wraps a JsonDoc whose root is an array of line values.
#[derive(Debug, Clone)]
pub struct JsonlDoc {
    inner: JsonDoc,
}

impl JsonlDoc {
    /// Parse JSONL input (one JSON value per non-empty line).
    pub fn parse(input: &str) -> Result<Self, String> {
        let mut elements = Vec::new();
        for (i, line) in input.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let val: serde_json::Value = serde_json::from_str(trimmed).map_err(|e| format!("line {}: {e}", i + 1))?;
            elements.push(val);
        }
        let array = Value::Array(elements);
        let json_str = serde_json::to_string(&array).map_err(|e| e.to_string())?;
        let inner = JsonDoc::parse(&json_str)?;
        Ok(Self { inner })
    }
}

impl StructuredDoc for JsonlDoc {
    fn root(&self) -> NodeId {
        self.inner.root()
    }
    fn children(&self, id: NodeId) -> &[NodeId] {
        self.inner.children(id)
    }
    fn node_kind(&self, id: NodeId) -> NodeKind {
        self.inner.node_kind(id)
    }
    fn key(&self, id: NodeId) -> Option<&str> {
        self.inner.key(id)
    }
    fn value_display(&self, id: NodeId) -> &str {
        self.inner.value_display(id)
    }
    fn meta(&self, id: NodeId) -> &str {
        self.inner.meta(id)
    }
    fn is_inline(&self, id: NodeId) -> bool {
        self.inner.is_inline(id)
    }
    fn is_expanded(&self, id: NodeId) -> bool {
        self.inner.is_expanded(id)
    }
    fn toggle_expand(&mut self, id: NodeId) {
        self.inner.toggle_expand(id);
    }
    fn set_key(&mut self, id: NodeId, key: &str) -> Result<(), String> {
        self.inner.set_key(id, key)
    }
    fn set_value(&mut self, id: NodeId, val: &str) -> Result<(), String> {
        self.inner.set_value(id, val)
    }
    fn set_meta(&mut self, id: NodeId, meta: &str) {
        self.inner.set_meta(id, meta);
    }
    fn toggle_inline(&mut self, id: NodeId) {
        self.inner.toggle_inline(id);
    }
    fn add_sibling(&mut self, id: NodeId) -> Result<NodeId, String> {
        self.inner.add_sibling(id)
    }
    fn add_child(&mut self, id: NodeId) -> Result<NodeId, String> {
        self.inner.add_child(id)
    }
    fn clone_node(&mut self, id: NodeId) -> Result<NodeId, String> {
        self.inner.clone_node(id)
    }
    fn remove(&mut self, id: NodeId) -> Result<(), String> {
        self.inner.remove(id)
    }
    fn swap_up(&mut self, id: NodeId) -> Result<(), String> {
        self.inner.swap_up(id)
    }
    fn swap_down(&mut self, id: NodeId) -> Result<(), String> {
        self.inner.swap_down(id)
    }
    fn promote(&mut self, id: NodeId) -> Result<(), String> {
        self.inner.promote(id)
    }
    fn demote(&mut self, id: NodeId) -> Result<(), String> {
        self.inner.demote(id)
    }
    fn cycle_type(&mut self, id: NodeId) {
        self.inner.cycle_type(id);
    }
    fn convert_container(&mut self, id: NodeId) {
        self.inner.convert_container(id);
    }
    fn sort_children(&mut self, id: NodeId, ascending: bool) {
        self.inner.sort_children(id, ascending);
    }
    fn sort_children_by_path(&mut self, id: NodeId, path: &str, ascending: bool) {
        self.inner.sort_children_by_path(id, path, ascending);
    }
    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.inner.parent(id)
    }
    fn scalar_type(&self, id: NodeId) -> ScalarType {
        self.inner.scalar_type(id)
    }

    /// Serialize as JSONL: one compact JSON value per top-level array element.
    fn serialize(&self) -> String {
        let mut out = String::new();
        let root = self.inner.root();
        for &child in self.inner.children(root) {
            let child_json = serialize_node_compact(&self.inner, child);
            out.push_str(&child_json);
            out.push('\n');
        }
        out
    }

    fn snapshot(&self) -> String {
        self.serialize()
    }

    fn restore(&mut self, snapshot: &str) -> Result<(), String> {
        let new_doc = JsonlDoc::parse(snapshot)?;
        self.inner = new_doc.inner;
        Ok(())
    }

    fn serialize_node(&self, id: NodeId) -> String {
        self.inner.serialize_node(id)
    }

    fn paste_after(&mut self, id: NodeId, json: &str) -> Result<NodeId, String> {
        self.inner.paste_after(id, json)
    }
}

/// Serialize a single node as compact JSON (used for JSONL line output).
fn serialize_node_compact(doc: &JsonDoc, id: NodeId) -> String {
    let node = doc.node(id);
    match node.kind {
        NodeKind::Scalar => serialize_scalar_compact(node),
        NodeKind::Dict => {
            let children = &node.children;
            if children.is_empty() {
                return "{}".to_string();
            }
            let mut out = String::from("{");
            for (i, &child) in children.iter().enumerate() {
                if let Some(k) = doc.node(child).key.as_deref() {
                    out.push('"');
                    out.push_str(&escape_json(k));
                    out.push_str("\":");
                }
                out.push_str(&serialize_node_compact(doc, child));
                if i < children.len() - 1 {
                    out.push(',');
                }
            }
            out.push('}');
            out
        }
        NodeKind::Array => {
            let children = &node.children;
            if children.is_empty() {
                return "[]".to_string();
            }
            let mut out = String::from("[");
            for (i, &child) in children.iter().enumerate() {
                out.push_str(&serialize_node_compact(doc, child));
                if i < children.len() - 1 {
                    out.push(',');
                }
            }
            out.push(']');
            out
        }
    }
}

fn serialize_scalar_compact(node: &crate::structured::json_doc::Node) -> String {
    match node.scalar_type {
        ScalarType::String => format!("\"{}\"", escape_json(&node.value)),
        _ => node.value.clone(),
    }
}

fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < '\x20' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
