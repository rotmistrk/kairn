use std::path::{Path, PathBuf};

/// A node in the file tree.
#[derive(Debug)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub kind: NodeKind,
}

#[derive(Debug)]
pub enum NodeKind {
    File,
    Dir {
        children: Vec<FileNode>,
        expanded: bool,
    },
}

impl FileNode {
    pub fn file(name: String, path: PathBuf) -> Self {
        Self {
            name,
            path,
            kind: NodeKind::File,
        }
    }

    pub fn dir(name: String, path: PathBuf, children: Vec<FileNode>) -> Self {
        Self {
            name,
            path,
            kind: NodeKind::Dir {
                children,
                expanded: false,
            },
        }
    }

    pub fn is_dir(&self) -> bool {
        matches!(self.kind, NodeKind::Dir { .. })
    }

    pub fn toggle_expanded(&mut self) {
        if let NodeKind::Dir { expanded, .. } = &mut self.kind {
            *expanded = !*expanded;
        }
    }

    pub fn set_expanded(&mut self, val: bool) {
        if let NodeKind::Dir { expanded, .. } = &mut self.kind {
            *expanded = val;
        }
    }
}

/// A flattened row for rendering — produced by walking the tree.
pub struct FlatEntry<'a> {
    pub node: &'a FileNode,
    pub depth: usize,
}

/// Walk the tree and produce a flat list of visible entries.
pub fn flatten(roots: &[FileNode]) -> Vec<FlatEntry<'_>> {
    let mut out = Vec::new();
    for node in roots {
        flatten_node(node, 0, &mut out);
    }
    out
}

fn flatten_node<'a>(node: &'a FileNode, depth: usize, out: &mut Vec<FlatEntry<'a>>) {
    out.push(FlatEntry { node, depth });
    if let NodeKind::Dir {
        children,
        expanded: true,
    } = &node.kind
    {
        for child in children {
            flatten_node(child, depth + 1, out);
        }
    }
}

/// Find a mutable reference to the node at the given flat index.
/// Returns None if index is out of bounds.
pub fn node_at_mut(roots: &mut [FileNode], flat_idx: usize) -> Option<&mut FileNode> {
    let mut counter = 0usize;
    for root in roots.iter_mut() {
        let result = node_at_mut_inner(root, flat_idx, &mut counter);
        if result.is_some() {
            return result;
        }
    }
    None
}

/// Find a node by its filesystem path.
pub fn node_by_path_mut<'a>(
    roots: &'a mut [FileNode],
    target: &std::path::Path,
) -> Option<&'a mut FileNode> {
    for root in roots.iter_mut() {
        if root.path == target {
            return Some(root);
        }
        if let NodeKind::Dir { children, .. } = &mut root.kind {
            let found = node_by_path_mut(children, target);
            if found.is_some() {
                return found;
            }
        }
    }
    None
}

fn node_at_mut_inner<'a>(
    node: &'a mut FileNode,
    target: usize,
    counter: &mut usize,
) -> Option<&'a mut FileNode> {
    if *counter == target {
        return Some(node);
    }
    *counter += 1;
    if let NodeKind::Dir {
        children,
        expanded: true,
    } = &mut node.kind
    {
        for child in children.iter_mut() {
            let result = node_at_mut_inner(child, target, counter);
            if result.is_some() {
                return result;
            }
        }
    }
    None
}

/// Build a tree from a root directory path using the `ignore` crate.
pub fn scan_workspace(root: &Path) -> anyhow::Result<Vec<FileNode>> {
    let mut top_children = Vec::new();
    scan_dir(root, &mut top_children)?;
    sort_nodes(&mut top_children);
    Ok(top_children)
}

fn scan_dir(dir: &Path, out: &mut Vec<FileNode>) -> anyhow::Result<()> {
    let walker = ignore::WalkBuilder::new(dir)
        .max_depth(Some(1))
        .hidden(true)
        .sort_by_file_name(|a, b| a.cmp(b))
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path().to_path_buf();

        // Skip the root directory itself
        if path == dir {
            continue;
        }

        let name = entry.file_name().to_string_lossy().to_string();

        if entry.file_type().is_some_and(|ft| ft.is_dir()) {
            let mut children = Vec::new();
            scan_dir(&path, &mut children)?;
            sort_nodes(&mut children);
            out.push(FileNode::dir(name, path, children));
        } else {
            out.push(FileNode::file(name, path));
        }
    }
    Ok(())
}

/// Dirs first, then files, alphabetical within each group.
fn sort_nodes(nodes: &mut [FileNode]) {
    nodes.sort_by(|a, b| {
        let a_dir = a.is_dir();
        let b_dir = b.is_dir();
        b_dir.cmp(&a_dir).then_with(|| a.name.cmp(&b.name))
    });
}
