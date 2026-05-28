//! Structural mutation operations for JsonDoc.

use std::cmp::Ordering;

use crate::structured::{NodeId, NodeKind, ScalarType};

use super::JsonDoc;

pub(crate) fn add_sibling(doc: &mut JsonDoc, id: NodeId) -> Result<NodeId, String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot add sibling to root")?;
    let new_id = doc.alloc_scalar(None, "null".into(), ScalarType::Null);
    doc.node_mut(new_id).parent = Some(parent_id);
    let siblings = &doc.nodes[parent_id.0].children;
    let pos = siblings.iter().position(|c| *c == id).unwrap_or(siblings.len());
    doc.nodes[parent_id.0].children.insert(pos + 1, new_id);
    doc.update_container_display(parent_id);
    Ok(new_id)
}

pub(crate) fn add_child(doc: &mut JsonDoc, id: NodeId) -> Result<NodeId, String> {
    let kind = doc.node(id).kind;
    if kind == NodeKind::Scalar {
        return Err("Cannot add child to scalar".into());
    }
    let new_id = doc.alloc_scalar(None, "null".into(), ScalarType::Null);
    doc.node_mut(new_id).parent = Some(id);
    doc.nodes[id.0].children.push(new_id);
    doc.update_container_display(id);
    Ok(new_id)
}

pub(crate) fn clone_node(doc: &mut JsonDoc, id: NodeId) -> Result<NodeId, String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot clone root")?;
    let cloned = deep_clone(doc, id);
    doc.node_mut(cloned).parent = Some(parent_id);
    let siblings = &doc.nodes[parent_id.0].children;
    let pos = siblings.iter().position(|c| *c == id).unwrap_or(siblings.len());
    doc.nodes[parent_id.0].children.insert(pos + 1, cloned);
    doc.update_container_display(parent_id);
    Ok(cloned)
}

fn deep_clone(doc: &mut JsonDoc, id: NodeId) -> NodeId {
    let node = doc.nodes[id.0].clone();
    let new_id = NodeId(doc.nodes.len());
    doc.nodes.push(node);
    doc.nodes[new_id.0].children.clear();
    let old_children: Vec<NodeId> = doc.nodes[id.0].children.clone();
    for child in old_children {
        let cloned_child = deep_clone(doc, child);
        doc.nodes[cloned_child.0].parent = Some(new_id);
        doc.nodes[new_id.0].children.push(cloned_child);
    }
    new_id
}

pub(crate) fn remove(doc: &mut JsonDoc, id: NodeId) -> Result<(), String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot remove root")?;
    doc.nodes[parent_id.0].children.retain(|c| *c != id);
    doc.update_container_display(parent_id);
    Ok(())
}

pub(crate) fn swap_up(doc: &mut JsonDoc, id: NodeId) -> Result<(), String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot swap root")?;
    let children = &doc.nodes[parent_id.0].children;
    let pos = children
        .iter()
        .position(|c| *c == id)
        .ok_or("Node not found in parent")?;
    if pos == 0 {
        return Err("Already first".into());
    }
    doc.nodes[parent_id.0].children.swap(pos, pos - 1);
    Ok(())
}

pub(crate) fn swap_down(doc: &mut JsonDoc, id: NodeId) -> Result<(), String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot swap root")?;
    let children = &doc.nodes[parent_id.0].children;
    let pos = children
        .iter()
        .position(|c| *c == id)
        .ok_or("Node not found in parent")?;
    let len = children.len();
    if pos >= len - 1 {
        return Err("Already last".into());
    }
    doc.nodes[parent_id.0].children.swap(pos, pos + 1);
    Ok(())
}

pub(crate) fn promote(doc: &mut JsonDoc, id: NodeId) -> Result<(), String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot promote root")?;
    let grandparent_id = doc.node(parent_id).parent.ok_or("Parent is root, cannot promote")?;
    // Remove from parent
    doc.nodes[parent_id.0].children.retain(|c| *c != id);
    doc.update_container_display(parent_id);
    // Insert after parent in grandparent
    let gp_children = &doc.nodes[grandparent_id.0].children;
    let parent_pos = gp_children.iter().position(|c| *c == parent_id).unwrap_or(0);
    doc.nodes[grandparent_id.0].children.insert(parent_pos + 1, id);
    doc.nodes[id.0].parent = Some(grandparent_id);
    doc.update_container_display(grandparent_id);
    Ok(())
}

pub(crate) fn demote(doc: &mut JsonDoc, id: NodeId) -> Result<(), String> {
    let parent_id = doc.node(id).parent.ok_or("Cannot demote root")?;
    let siblings = &doc.nodes[parent_id.0].children;
    let pos = siblings.iter().position(|c| *c == id).ok_or("Node not found")?;
    if pos == 0 {
        return Err("No previous sibling to demote into".into());
    }
    let prev_sibling = siblings[pos - 1];
    if doc.node(prev_sibling).kind == NodeKind::Scalar {
        return Err("Previous sibling is scalar, cannot demote into it".into());
    }
    // Remove from parent
    doc.nodes[parent_id.0].children.retain(|c| *c != id);
    doc.update_container_display(parent_id);
    // Add as last child of previous sibling
    doc.nodes[prev_sibling.0].children.push(id);
    doc.nodes[id.0].parent = Some(prev_sibling);
    doc.update_container_display(prev_sibling);
    Ok(())
}

pub(crate) fn cycle_type(doc: &mut JsonDoc, id: NodeId) {
    if doc.node(id).kind != NodeKind::Scalar {
        return;
    }
    let (new_type, new_val) = match doc.node(id).scalar_type {
        ScalarType::String => (ScalarType::Number, "0".into()),
        ScalarType::Number => (ScalarType::Bool, "true".into()),
        ScalarType::Bool => (ScalarType::Null, "null".into()),
        ScalarType::Null => (ScalarType::String, String::new()),
    };
    doc.nodes[id.0].scalar_type = new_type;
    doc.nodes[id.0].value = new_val;
}

pub(crate) fn convert_container(doc: &mut JsonDoc, id: NodeId) {
    let kind = doc.node(id).kind;
    match kind {
        NodeKind::Dict => {
            doc.nodes[id.0].kind = NodeKind::Array;
            let children: Vec<NodeId> = doc.nodes[id.0].children.clone();
            for child in children {
                doc.nodes[child.0].key = None;
            }
        }
        NodeKind::Array => {
            doc.nodes[id.0].kind = NodeKind::Dict;
            let children: Vec<NodeId> = doc.nodes[id.0].children.clone();
            for (i, child) in children.iter().enumerate() {
                doc.nodes[child.0].key = Some(format!("key{i}"));
            }
        }
        NodeKind::Scalar => return,
    }
    doc.update_container_display(id);
}

pub(crate) fn sort_children(doc: &mut JsonDoc, id: NodeId, ascending: bool) {
    let kind = doc.node(id).kind;
    if kind == NodeKind::Scalar {
        return;
    }
    let mut children: Vec<NodeId> = doc.nodes[id.0].children.clone();
    match kind {
        NodeKind::Dict => sort_dict_children(doc, &mut children, ascending),
        NodeKind::Array => sort_array_children(doc, &mut children, ascending),
        _ => {}
    }
    doc.nodes[id.0].children = children;
}

fn sort_dict_children(doc: &JsonDoc, children: &mut [NodeId], ascending: bool) {
    children.sort_by(|a, b| {
        let ka = doc.nodes[a.0].key.as_deref().unwrap_or("");
        let kb = doc.nodes[b.0].key.as_deref().unwrap_or("");
        if ascending {
            ka.cmp(kb)
        } else {
            kb.cmp(ka)
        }
    });
}

fn sort_array_children(doc: &JsonDoc, children: &mut [NodeId], ascending: bool) {
    let all_numeric = children.iter().all(|c| doc.nodes[c.0].value.parse::<f64>().is_ok());
    if all_numeric {
        children.sort_by(|a, b| {
            let va = doc.nodes[a.0].value.parse::<f64>().unwrap_or(0.0);
            let vb = doc.nodes[b.0].value.parse::<f64>().unwrap_or(0.0);
            let cmp = va.partial_cmp(&vb).unwrap_or(Ordering::Equal);
            if ascending {
                cmp
            } else {
                cmp.reverse()
            }
        });
    } else {
        children.sort_by(|a, b| {
            let va = &doc.nodes[a.0].value;
            let vb = &doc.nodes[b.0].value;
            if ascending {
                va.cmp(vb)
            } else {
                vb.cmp(va)
            }
        });
    }
}

pub(crate) fn sort_children_by_path(doc: &mut JsonDoc, id: NodeId, path: &str, ascending: bool) {
    if doc.node(id).kind != NodeKind::Array {
        return;
    }
    let segments: Vec<&str> = path
        .trim_start_matches('.')
        .split('.')
        .filter(|s| !s.is_empty())
        .collect();
    let mut children: Vec<NodeId> = doc.nodes[id.0].children.clone();
    children.sort_by(|a, b| {
        let va = resolve_path(doc, *a, &segments);
        let vb = resolve_path(doc, *b, &segments);
        let cmp = if let (Some(fa), Some(fb)) = (
            va.as_deref().and_then(|s| s.parse::<f64>().ok()),
            vb.as_deref().and_then(|s| s.parse::<f64>().ok()),
        ) {
            fa.partial_cmp(&fb).unwrap_or(Ordering::Equal)
        } else {
            let sa = va.as_deref().unwrap_or("");
            let sb = vb.as_deref().unwrap_or("");
            sa.cmp(sb)
        };
        if ascending {
            cmp
        } else {
            cmp.reverse()
        }
    });
    doc.nodes[id.0].children = children;
}

fn resolve_path(doc: &JsonDoc, id: NodeId, segments: &[&str]) -> Option<String> {
    let mut current = id;
    for &seg in segments {
        let children = &doc.nodes[current.0].children;
        let found = children.iter().find(|c| doc.nodes[c.0].key.as_deref() == Some(seg));
        current = *found?;
    }
    Some(doc.nodes[current.0].value.clone())
}
