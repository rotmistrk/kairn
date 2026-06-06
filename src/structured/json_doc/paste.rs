//! Paste (deep-copy from parsed JSON) for JsonDoc.

use crate::structured::{NodeId, NodeKind, StructuredDoc};

use super::JsonDoc;

pub(crate) fn paste_after(doc: &mut JsonDoc, id: NodeId, json: &str) -> Result<NodeId, String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot paste at root level")?;
    let parent_kind = doc.node(parent_id).kind;

    let parsed = JsonDoc::parse(json).or_else(|_| JsonDoc::parse(&format!("[{json}]")))?;

    let source_root = parsed.root();
    let source_children = &parsed.nodes[source_root.0].children;

    let source_id = if (parent_kind == NodeKind::Dict && parsed.node(source_root).kind == NodeKind::Dict)
        || (parent_kind == NodeKind::Array && parsed.node(source_root).kind == NodeKind::Array)
    {
        source_children.first().copied().unwrap_or(source_root)
    } else {
        source_root
    };

    let new_id = deep_copy_node(doc, &parsed, source_id, Some(parent_id));

    let children = &doc.nodes[parent_id.0].children;
    let pos = children.iter().position(|c| *c == id).unwrap_or(children.len());
    doc.nodes[parent_id.0].children.insert(pos + 1, new_id);
    doc.update_container_display(parent_id);
    Ok(new_id)
}

fn deep_copy_node(doc: &mut JsonDoc, source: &JsonDoc, source_id: NodeId, parent: Option<NodeId>) -> NodeId {
    let src = source.node(source_id);
    let new_id = NodeId(doc.nodes.len());
    doc.nodes.push(super::Node {
        kind: src.kind,
        key: src.key.clone(),
        value: src.value.clone(),
        scalar_type: src.scalar_type,
        children: Vec::new(),
        parent,
        expanded: src.expanded,
        inline: src.inline,
        meta: src.meta.clone(),
    });
    let child_ids: Vec<NodeId> = source
        .node(source_id)
        .children
        .iter()
        .map(|&child_id| deep_copy_node(doc, source, child_id, Some(new_id)))
        .collect();
    doc.nodes[new_id.0].children = child_ids;
    new_id
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structured::StructuredDoc;

    #[test]
    fn test_paste_node() {
        let mut doc = JsonDoc::parse(r#"{"items":["alpha","beta","gamma"]}"#).unwrap();
        let root = doc.root();
        let items_id = doc.children(root)[0];
        let first = doc.children(items_id)[0];
        let json = doc.serialize_node(first);
        let last = doc.children(items_id)[2];
        let result = paste_after(&mut doc, last, &json);
        assert!(result.is_ok(), "paste should succeed: {:?}", result);
        assert_eq!(doc.children(items_id).len(), 4);
    }
}
