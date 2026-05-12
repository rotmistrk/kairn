//! JSON implementation of StructuredDoc.

mod node_lines;
mod ops;
mod serialize;

use crate::structured::jsonc_parse;
use crate::structured::{NodeId, NodeKind, ScalarType, StructuredDoc};

/// A single node in the JSON document arena.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub key: Option<String>,
    pub value: String,
    pub meta: String,
    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,
    pub expanded: bool,
    pub inline: bool,
    pub scalar_type: ScalarType,
}

/// Arena-backed JSON document.
#[derive(Debug, Clone)]
pub struct JsonDoc {
    pub(crate) nodes: Vec<Node>,
}

impl JsonDoc {
    /// Parse a JSON string into a document tree.
    pub fn parse(input: &str) -> Result<Self, String> {
        let val: serde_json::Value = serde_json::from_str(input).map_err(|e| e.to_string())?;
        let mut doc = Self { nodes: Vec::new() };
        doc.build_node(&val, None);
        Ok(doc)
    }

    /// Parse JSONC (JSON with comments) into a document tree, preserving comments in node meta.
    pub fn parse_jsonc(input: &str) -> Result<Self, String> {
        let (stripped, comments) = jsonc_parse::strip_comments(input);
        let mut doc = Self::parse(&stripped)?;
        if comments.is_empty() {
            return Ok(doc);
        }
        // Assign comments to nodes by line proximity: each comment attaches to the
        // first DFS-order node whose line >= comment line in the stripped source.
        let node_lines = node_lines::compute_node_lines(&stripped, &doc);
        for comment in &comments {
            let prefix = match comment.kind {
                jsonc_parse::CommentKind::Line => "//",
                jsonc_parse::CommentKind::Block => "/*",
            };
            let text = format!("{prefix} {}", comment.text);
            // Find first node at or after this comment's line
            let target = node_lines
                .iter()
                .filter(|(_, line)| *line >= comment.line)
                .min_by_key(|(_, line)| *line)
                .map(|(id, _)| *id);
            if let Some(id) = target {
                let existing = &doc.nodes[id.0].meta;
                if existing.is_empty() {
                    doc.nodes[id.0].meta = text;
                } else {
                    doc.nodes[id.0].meta = format!("{existing}\n{text}");
                }
            }
        }
        Ok(doc)
    }

    fn build_node(&mut self, val: &serde_json::Value, key: Option<String>) -> NodeId {
        match val {
            serde_json::Value::Object(map) => {
                let id = self.alloc(NodeKind::Dict, key, format!("{{{}}}", map.len()));
                for (k, v) in map {
                    let child = self.build_node(v, Some(k.clone()));
                    self.nodes[child.0].parent = Some(id);
                    self.nodes[id.0].children.push(child);
                }
                id
            }
            serde_json::Value::Array(arr) => {
                let id = self.alloc(NodeKind::Array, key, format!("[{}]", arr.len()));
                for item in arr {
                    let child = self.build_node(item, None);
                    self.nodes[child.0].parent = Some(id);
                    self.nodes[id.0].children.push(child);
                }
                id
            }
            _ => {
                let (display, scalar_type) = Self::scalar_info(val);
                self.alloc_scalar(key, display, scalar_type)
            }
        }
    }

    fn scalar_info(val: &serde_json::Value) -> (String, ScalarType) {
        match val {
            serde_json::Value::Null => ("null".into(), ScalarType::Null),
            serde_json::Value::Bool(b) => (b.to_string(), ScalarType::Bool),
            serde_json::Value::Number(n) => (n.to_string(), ScalarType::Number),
            serde_json::Value::String(s) => (s.clone(), ScalarType::String),
            _ => ("null".into(), ScalarType::Null),
        }
    }

    fn alloc(&mut self, kind: NodeKind, key: Option<String>, value: String) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            kind,
            key,
            value,
            meta: String::new(),
            children: Vec::new(),
            parent: None,
            expanded: true,
            inline: false,
            scalar_type: ScalarType::Null,
        });
        id
    }

    fn alloc_scalar(&mut self, key: Option<String>, value: String, scalar_type: ScalarType) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(Node {
            kind: NodeKind::Scalar,
            key,
            value,
            meta: String::new(),
            children: Vec::new(),
            parent: None,
            expanded: true,
            inline: false,
            scalar_type,
        });
        id
    }

    pub(crate) fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub(crate) fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    fn update_container_display(&mut self, id: NodeId) {
        let n = &self.nodes[id.0];
        let count = n.children.len();
        let display = match n.kind {
            NodeKind::Dict => format!("{{{count}}}"),
            NodeKind::Array => format!("[{count}]"),
            NodeKind::Scalar => return,
        };
        self.nodes[id.0].value = display;
    }
}

impl StructuredDoc for JsonDoc {
    fn root(&self) -> NodeId {
        NodeId(0)
    }
    fn children(&self, id: NodeId) -> &[NodeId] {
        &self.nodes[id.0].children
    }
    fn node_kind(&self, id: NodeId) -> NodeKind {
        self.nodes[id.0].kind
    }
    fn key(&self, id: NodeId) -> Option<&str> {
        self.nodes[id.0].key.as_deref()
    }
    fn value_display(&self, id: NodeId) -> &str {
        &self.nodes[id.0].value
    }
    fn meta(&self, id: NodeId) -> &str {
        &self.nodes[id.0].meta
    }
    fn is_inline(&self, id: NodeId) -> bool {
        self.nodes[id.0].inline
    }
    fn is_expanded(&self, id: NodeId) -> bool {
        self.nodes[id.0].expanded
    }
    fn toggle_expand(&mut self, id: NodeId) {
        self.nodes[id.0].expanded = !self.nodes[id.0].expanded;
    }
    fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.nodes[id.0].parent
    }
    fn scalar_type(&self, id: NodeId) -> ScalarType {
        self.nodes[id.0].scalar_type
    }

    fn set_key(&mut self, id: NodeId, key: &str) -> Result<(), String> {
        self.nodes[id.0].key = Some(key.to_string());
        Ok(())
    }

    fn set_value(&mut self, id: NodeId, val: &str) -> Result<(), String> {
        if self.nodes[id.0].kind != NodeKind::Scalar {
            return Err("Cannot set value on container".into());
        }
        self.nodes[id.0].value = val.to_string();
        Ok(())
    }

    fn set_meta(&mut self, id: NodeId, meta: &str) {
        self.nodes[id.0].meta = meta.to_string();
    }

    fn toggle_inline(&mut self, id: NodeId) {
        self.nodes[id.0].inline = !self.nodes[id.0].inline;
    }

    // Structural mutations delegated to ops module
    fn add_sibling(&mut self, id: NodeId) -> Result<NodeId, String> {
        ops::add_sibling(self, id)
    }
    fn add_child(&mut self, id: NodeId) -> Result<NodeId, String> {
        ops::add_child(self, id)
    }
    fn clone_node(&mut self, id: NodeId) -> Result<NodeId, String> {
        ops::clone_node(self, id)
    }
    fn remove(&mut self, id: NodeId) -> Result<(), String> {
        ops::remove(self, id)
    }
    fn swap_up(&mut self, id: NodeId) -> Result<(), String> {
        ops::swap_up(self, id)
    }
    fn swap_down(&mut self, id: NodeId) -> Result<(), String> {
        ops::swap_down(self, id)
    }
    fn promote(&mut self, id: NodeId) -> Result<(), String> {
        ops::promote(self, id)
    }
    fn demote(&mut self, id: NodeId) -> Result<(), String> {
        ops::demote(self, id)
    }
    fn cycle_type(&mut self, id: NodeId) {
        ops::cycle_type(self, id);
    }
    fn convert_container(&mut self, id: NodeId) {
        ops::convert_container(self, id);
    }
    fn sort_children(&mut self, id: NodeId, ascending: bool) {
        ops::sort_children(self, id, ascending);
    }
    fn sort_children_by_path(&mut self, id: NodeId, path: &str, ascending: bool) {
        ops::sort_children_by_path(self, id, path, ascending);
    }
    fn serialize(&self) -> String {
        serialize::serialize(self)
    }
    fn snapshot(&self) -> String {
        serialize::serialize(self)
    }
    fn restore(&mut self, snapshot: &str) -> Result<(), String> {
        let new_doc = JsonDoc::parse(snapshot)?;
        self.nodes = new_doc.nodes;
        Ok(())
    }
}
