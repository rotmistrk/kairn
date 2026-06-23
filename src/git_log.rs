//! Git log — async commit history loading via git2 Revwalk.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use git2::{Oid, Repository, Sort};
use txv_core::cell::Color;

/// A single commit entry for display.
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub(crate) hash: String,
    pub(crate) summary: String,
    pub(crate) author: String,
    pub(crate) time_secs: i64,
    pub(crate) decorations: Vec<String>,
    pub(crate) root: PathBuf,
    pub(crate) root_color: Color,
}

/// State of the log loading.
#[derive(Debug, Clone)]
pub enum LogState {
    Loading,
    Ready(Vec<CommitEntry>),
    Error(String),
}

/// Shared log result, written by background thread.
pub type SharedLog = Arc<Mutex<LogState>>;

/// Load commit log asynchronously.
/// `filter_path`: if Some, only show commits touching that file.
/// `branch`: if Some, start walk from that branch; if "--all", walk all refs.
pub fn log_async(root: &Path, branch: Option<&str>, filter_path: Option<&Path>) -> SharedLog {
    let roots = vec![(root.to_path_buf(), Color::Reset)];
    log_async_roots(&roots, branch, filter_path)
}

/// Load commit log from multiple roots, merged by time descending.
pub fn log_async_roots(roots: &[(PathBuf, Color)], branch: Option<&str>, filter_path: Option<&Path>) -> SharedLog {
    let state: SharedLog = Arc::new(Mutex::new(LogState::Loading));
    let state_clone = Arc::clone(&state);
    let roots = roots.to_vec();
    let branch = branch.map(String::from);
    let filter = filter_path.map(|p| p.to_path_buf());

    thread::spawn(move || {
        let result = compute_log_multi(&roots, branch.as_deref(), filter.as_deref());
        if let Ok(mut guard) = state_clone.lock() {
            *guard = result;
        }
    });

    state
}

fn compute_log_multi(roots: &[(PathBuf, Color)], branch: Option<&str>, filter_path: Option<&Path>) -> LogState {
    let mut all_entries = Vec::new();
    for (root, color) in roots {
        match compute_log(root, branch, filter_path, *color) {
            LogState::Ready(entries) => all_entries.extend(entries),
            LogState::Error(e) if roots.len() == 1 => return LogState::Error(e),
            _ => {}
        }
    }
    all_entries.sort_by(|a, b| b.time_secs.cmp(&a.time_secs));
    all_entries.truncate(200);
    LogState::Ready(all_entries)
}

fn compute_log(root: &Path, branch: Option<&str>, filter_path: Option<&Path>, root_color: Color) -> LogState {
    let repo = match Repository::discover(root) {
        Ok(r) => r,
        Err(e) => return LogState::Error(format!("Not a git repo: {e}")),
    };
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(e) => return LogState::Error(format!("Revwalk failed: {e}")),
    };
    revwalk.set_sorting(Sort::TIME).ok();

    if let Err(e) = push_start(&repo, &mut revwalk, branch) {
        return LogState::Error(e);
    }

    let decorations = build_decoration_map(&repo);
    let entries = collect_entries(&repo, revwalk, &decorations, filter_path, root, root_color);
    LogState::Ready(entries)
}

fn collect_entries(
    repo: &Repository,
    revwalk: git2::Revwalk,
    decorations: &HashMap<Oid, Vec<String>>,
    filter_path: Option<&Path>,
    root: &Path,
    root_color: Color,
) -> Vec<CommitEntry> {
    let mut entries = Vec::new();
    let limit = 200;

    for oid_result in revwalk {
        let Ok(oid) = oid_result else {
            continue;
        };
        let Ok(commit) = repo.find_commit(oid) else {
            continue;
        };

        if let Some(fp) = filter_path {
            if !commit_touches_path(repo, &commit, fp) {
                continue;
            }
        }

        entries.push(make_entry(&commit, oid, decorations, root, root_color));
        if entries.len() >= limit {
            break;
        }
    }
    entries
}

fn make_entry(
    commit: &git2::Commit,
    oid: Oid,
    decorations: &HashMap<Oid, Vec<String>>,
    root: &Path,
    root_color: Color,
) -> CommitEntry {
    let hash = format!("{}", oid)[..7].to_string();
    let summary = commit.summary().unwrap_or("").to_string();
    let author = commit.author().name().unwrap_or("?").to_string();
    let time_secs = commit.time().seconds();
    let decor = decorations.get(&oid).cloned().unwrap_or_default();
    CommitEntry {
        hash,
        summary,
        author,
        time_secs,
        decorations: decor,
        root: root.to_path_buf(),
        root_color,
    }
}

fn push_start(repo: &git2::Repository, revwalk: &mut git2::Revwalk, branch: Option<&str>) -> Result<(), String> {
    match branch {
        Some("--all") => {
            revwalk.push_glob("refs/*").map_err(|e| format!("push_glob: {e}"))?;
        }
        Some(name) => {
            let obj = repo
                .revparse_single(name)
                .map_err(|e| format!("Unknown ref '{name}': {e}"))?;
            revwalk.push(obj.id()).map_err(|e| format!("push: {e}"))?;
        }
        None => {
            revwalk.push_head().map_err(|e| format!("push_head: {e}"))?;
        }
    }
    Ok(())
}

fn build_decoration_map(repo: &git2::Repository) -> HashMap<Oid, Vec<String>> {
    let mut map: HashMap<Oid, Vec<String>> = HashMap::new();
    if let Ok(refs) = repo.references() {
        for r in refs.flatten() {
            if let Some(target) = r.target() {
                let name = r.shorthand().unwrap_or("?").to_string();
                map.entry(target).or_default().push(name);
            }
        }
    }
    if let Ok(head) = repo.head() {
        if let Some(target) = head.target() {
            let branch_name = head.shorthand().unwrap_or("HEAD").to_string();
            let label = format!("HEAD -> {branch_name}");
            map.entry(target).or_default().insert(0, label);
        }
    }
    map
}

fn commit_touches_path(repo: &git2::Repository, commit: &git2::Commit, path: &Path) -> bool {
    let Ok(tree) = commit.tree() else {
        return false;
    };
    if commit.parent_count() == 0 {
        return tree.get_path(path).is_ok();
    }
    for i in 0..commit.parent_count() {
        let Ok(parent) = commit.parent(i) else {
            continue;
        };
        let Ok(parent_tree) = parent.tree() else {
            continue;
        };
        let Ok(diff) = repo.diff_tree_to_tree(Some(&parent_tree), Some(&tree), None) else {
            continue;
        };
        if diff_contains_path(&diff, path) {
            return true;
        }
    }
    false
}

fn diff_contains_path(diff: &git2::Diff, path: &Path) -> bool {
    diff.deltas().any(|delta| delta.new_file().path() == Some(path))
}
