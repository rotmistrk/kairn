//! Builder helpers for git changes tree nodes.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use git2::Repository;
use txv_core::cell::Color;

use crate::app_palette::app_palette;
use crate::git_status::{collect_git_status, FileStatus};

use super::change_node::ChangeNode;

pub(super) fn collect_statuses_flat(root: &Path, git_roots: &[PathBuf]) -> HashMap<FileStatus, Vec<(String, PathBuf)>> {
    let mut by_status: HashMap<FileStatus, Vec<(String, PathBuf)>> = HashMap::new();
    for git_root in git_roots {
        let statuses = collect_git_status(git_root);
        for (rel_path, status) in statuses {
            if status == FileStatus::Clean || status == FileStatus::Ignored {
                continue;
            }
            let abs_path = git_root.join(&rel_path);
            let label = abs_path
                .strip_prefix(root)
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or(rel_path);
            by_status.entry(status).or_default().push((label, abs_path));
        }
    }
    by_status
}

pub(super) fn build_category_nodes(
    nodes: &mut Vec<ChangeNode>,
    by_status: &HashMap<FileStatus, Vec<(String, PathBuf)>>,
    cat_depth: usize,
    file_depth: usize,
    root: &Path,
) {
    let app = app_palette();
    let categories = [
        (FileStatus::Conflict, "Conflicts", app.git().conflict().fg()),
        (FileStatus::Modified, "Modified", app.git().modified().fg()),
        (FileStatus::Added, "Added", app.git().added().fg()),
        (FileStatus::Untracked, "Untracked", app.git().untracked().fg()),
    ];
    let root_str = root.to_string_lossy();
    for (status, name, color) in &categories {
        let Some(files) = by_status.get(status) else {
            continue;
        };
        if files.is_empty() {
            continue;
        }
        let label = format!("{name} ({})", files.len());
        nodes.push(ChangeNode {
            label,
            depth: cat_depth,
            expandable: true,
            expanded: true,
            file_path: None,
            color: *color,
            status: None,
            key: Some(format!("{root_str}:{name}")),
        });
        push_file_nodes(nodes, files, file_depth, *color, *status);
    }
}

fn push_file_nodes(
    nodes: &mut Vec<ChangeNode>,
    files: &[(String, PathBuf)],
    depth: usize,
    color: Color,
    status: FileStatus,
) {
    let mut sorted = files.to_vec();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));
    for (rel, abs) in sorted {
        nodes.push(ChangeNode {
            label: rel,
            depth,
            expandable: false,
            expanded: false,
            file_path: Some(abs),
            color,
            status: Some(status),
            key: None,
        });
    }
}

/// Discover all git roots under the workspace root.
pub(super) fn discover_git_roots(root: &Path) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if root.join(".git").exists() {
        roots.push(root.to_path_buf());
    }
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() && path != root {
                roots.push(path);
            }
        }
    }
    if roots.is_empty() {
        if let Ok(repo) = Repository::discover(root) {
            if let Some(workdir) = repo.workdir() {
                roots.push(workdir.to_path_buf());
            }
        }
    }
    roots
}
