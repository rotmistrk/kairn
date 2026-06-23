//! Async git status collection — runs on background thread, pollable from UI.

use std::mem;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use txv_core::run::Waker;

use crate::app_palette::app_palette;
use crate::git_status_params::GitStatusParams;
use crate::views::git_changes::builders::{
    build_category_nodes, collect_diff_base_statuses, collect_statuses_flat, discover_git_roots,
};
use crate::views::git_changes::change_node::ChangeNode;

/// Shared state for an in-flight git status task.
pub(crate) struct GitStatusTask {
    nodes: Mutex<Vec<ChangeNode>>,
    done: AtomicBool,
    cancelled: AtomicBool,
}

impl GitStatusTask {
    pub(crate) fn is_done(&self) -> bool {
        self.done.load(Ordering::Relaxed)
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }

    pub(crate) fn take_nodes(&self) -> Vec<ChangeNode> {
        self.nodes.lock().map(|mut v| mem::take(&mut *v)).unwrap_or_default()
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }
}

/// Spawn async git status collection. Returns shared task handle.
pub(crate) fn git_status_async(params: GitStatusParams, waker: Waker) -> Arc<GitStatusTask> {
    let task = Arc::new(GitStatusTask {
        nodes: Mutex::new(Vec::new()),
        done: AtomicBool::new(false),
        cancelled: AtomicBool::new(false),
    });
    let task_clone = Arc::clone(&task);

    thread::spawn(move || {
        let nodes = compute_git_nodes(&params, &task_clone);
        if !task_clone.is_cancelled() {
            if let Ok(mut v) = task_clone.nodes.lock() {
                *v = nodes;
            }
            task_clone.done.store(true, Ordering::Relaxed);
            waker.wake();
        }
    });

    task
}

fn compute_git_nodes(params: &GitStatusParams, task: &GitStatusTask) -> Vec<ChangeNode> {
    let mut nodes = Vec::new();
    if params.roots.len() > 1 {
        compute_multi_root(&mut nodes, params, task);
    } else {
        compute_single_root(&mut nodes, params, task);
    }
    nodes
}

fn compute_single_root(nodes: &mut Vec<ChangeNode>, params: &GitStatusParams, task: &GitStatusTask) {
    let git_roots: Vec<PathBuf> = params.roots.iter().flat_map(|r| discover_git_roots(r)).collect();
    if task.is_cancelled() {
        return;
    }
    let parent = params.roots.first().map(|r| r.as_path()).unwrap_or(Path::new("."));
    let root = params.roots.first().cloned().unwrap_or_default();
    let base = params.diff_base.get(&root).map(|s| s.as_str());
    let by_status = if let Some(b) = base {
        collect_diff_base_statuses(parent, &git_roots, b)
    } else {
        collect_statuses_flat(parent, &git_roots)
    };
    if task.is_cancelled() {
        return;
    }
    build_category_nodes(nodes, &by_status, 0, 1, &root);
}

fn compute_multi_root(nodes: &mut Vec<ChangeNode>, params: &GitStatusParams, task: &GitStatusTask) {
    let app = app_palette();
    for (root_idx, root) in params.roots.iter().enumerate() {
        if task.is_cancelled() {
            return;
        }
        let git_roots = discover_git_roots(root);
        if git_roots.is_empty() {
            continue;
        }
        let base = params.diff_base.get(root).map(|s| s.as_str());
        let by_status = collect_for_root(root, &git_roots, base);
        if task.is_cancelled() || by_status.is_empty() {
            continue;
        }
        push_root_header(nodes, params, root_idx, root, base, &app);
        build_category_nodes(nodes, &by_status, 1, 2, root);
    }
}

fn collect_for_root(
    root: &Path,
    git_roots: &[PathBuf],
    base: Option<&str>,
) -> std::collections::HashMap<crate::git_status::FileStatus, Vec<(String, PathBuf)>> {
    if let Some(b) = base {
        collect_diff_base_statuses(root, git_roots, b)
    } else {
        collect_statuses_flat(root, git_roots)
    }
}

fn push_root_header(
    nodes: &mut Vec<ChangeNode>,
    params: &GitStatusParams,
    idx: usize,
    root: &Path,
    base: Option<&str>,
    app: &crate::app_palette::AppPalette,
) {
    let mut name = params.root_labels.get(idx).cloned().unwrap_or_else(|| {
        root.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| root.to_string_lossy().to_string())
    });
    if let Some(hash) = base {
        name.push_str(&format!(" [{hash}]"));
    }
    let color = params
        .root_badge_colors
        .get(idx)
        .copied()
        .unwrap_or(app.git().modified().fg());
    let expanded = !params.collapsed.contains(&root.to_string_lossy().to_string());
    nodes.push(ChangeNode {
        label: name,
        depth: 0,
        expandable: true,
        expanded,
        file_path: None,
        color,
        status: None,
        key: Some(root.to_string_lossy().to_string()),
    });
}
