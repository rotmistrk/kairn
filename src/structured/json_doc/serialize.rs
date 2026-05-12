//! JSON serialization for JsonDoc.

use crate::structured::{NodeId, NodeKind, ScalarType, StructuredDoc};

use super::JsonDoc;

pub(crate) fn serialize(doc: &JsonDoc) -> String {
    let mut out = String::new();
    write_node(doc, doc.root(), &mut out, 0);
    out.push('\n');
    out
}

fn write_node(doc: &JsonDoc, id: NodeId, out: &mut String, indent: usize) {
    let node = doc.node(id);
    match node.kind {
        NodeKind::Scalar => write_scalar(doc, id, out),
        NodeKind::Dict => write_dict(doc, id, out, indent),
        NodeKind::Array => write_array(doc, id, out, indent),
    }
}

fn write_scalar(doc: &JsonDoc, id: NodeId, out: &mut String) {
    let node = doc.node(id);
    match node.scalar_type {
        ScalarType::String => {
            out.push('"');
            out.push_str(&escape_json_string(&node.value));
            out.push('"');
        }
        _ => out.push_str(&node.value),
    }
}

fn write_dict(doc: &JsonDoc, id: NodeId, out: &mut String, indent: usize) {
    let children = &doc.node(id).children;
    if children.is_empty() {
        out.push_str("{}");
        return;
    }
    if doc.node(id).inline {
        write_dict_inline(doc, id, out);
        return;
    }
    out.push_str("{\n");
    for (i, &child) in children.iter().enumerate() {
        let child_indent = indent + 2;
        push_indent(out, child_indent);
        if let Some(k) = doc.node(child).key.as_deref() {
            out.push('"');
            out.push_str(&escape_json_string(k));
            out.push_str("\": ");
        }
        write_node(doc, child, out, child_indent);
        if i < children.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    push_indent(out, indent);
    out.push('}');
}

fn write_dict_inline(doc: &JsonDoc, id: NodeId, out: &mut String) {
    let children = &doc.node(id).children;
    out.push('{');
    for (i, &child) in children.iter().enumerate() {
        if let Some(k) = doc.node(child).key.as_deref() {
            out.push('"');
            out.push_str(&escape_json_string(k));
            out.push_str("\": ");
        }
        write_scalar_or_inline(doc, child, out);
        if i < children.len() - 1 {
            out.push_str(", ");
        }
    }
    out.push('}');
}

fn write_array(doc: &JsonDoc, id: NodeId, out: &mut String, indent: usize) {
    let children = &doc.node(id).children;
    if children.is_empty() {
        out.push_str("[]");
        return;
    }
    if doc.node(id).inline {
        write_array_inline(doc, id, out);
        return;
    }
    out.push_str("[\n");
    for (i, &child) in children.iter().enumerate() {
        let child_indent = indent + 2;
        push_indent(out, child_indent);
        write_node(doc, child, out, child_indent);
        if i < children.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
    push_indent(out, indent);
    out.push(']');
}

fn write_array_inline(doc: &JsonDoc, id: NodeId, out: &mut String) {
    let children = &doc.node(id).children;
    out.push('[');
    for (i, &child) in children.iter().enumerate() {
        write_scalar_or_inline(doc, child, out);
        if i < children.len() - 1 {
            out.push_str(", ");
        }
    }
    out.push(']');
}

fn write_scalar_or_inline(doc: &JsonDoc, id: NodeId, out: &mut String) {
    let node = doc.node(id);
    match node.kind {
        NodeKind::Scalar => write_scalar(doc, id, out),
        NodeKind::Dict => write_dict_inline(doc, id, out),
        NodeKind::Array => write_array_inline(doc, id, out),
    }
}

fn push_indent(out: &mut String, n: usize) {
    for _ in 0..n {
        out.push(' ');
    }
}

fn escape_json_string(s: &str) -> String {
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
