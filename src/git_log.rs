//! Git log — async commit history loading via git2 Revwalk.

use std::path::Path;
use std::sync::{Arc, Mutex};

/// A single commit entry for display.
#[derive(Debug, Clone)]
pub struct CommitEntry {
    pub(crate) hash: String,
    pub(crate) summary: String,
    pub(crate) author: String,
    pub(crate) time_secs: i64,
    pub(crate) decorations: Vec<String>,
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
    let state: SharedLog = Arc::new(Mutex::new(LogState::Loading));
    let state_clone = Arc::clone(&state);
    let root = root.to_path_buf();
    let branch = branch.map(String::from);
    let filter = filter_path.map(|p| p.to_path_buf());

    std::thread::spawn(move || {
        let result = compute_log(&root, branch.as_deref(), filter.as_deref());
        if let Ok(mut guard) = state_clone.lock() {
            *guard = result;
        }
    });

    state
}

fn compute_log(root: &Path, branch: Option<&str>, filter_path: Option<&Path>) -> LogState {
    let repo = match git2::Repository::discover(root) {
        Ok(r) => r,
        Err(e) => return LogState::Error(format!("Not a git repo: {e}")),
    };
    let mut revwalk = match repo.revwalk() {
        Ok(r) => r,
        Err(e) => return LogState::Error(format!("Revwalk failed: {e}")),
    };
    revwalk.set_sorting(git2::Sort::TIME).ok();

    if let Err(e) = push_start(&repo, &mut revwalk, branch) {
        return LogState::Error(e);
    }

    let decorations = build_decoration_map(&repo);
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
            if !commit_touches_path(&repo, &commit, fp) {
                continue;
            }
        }

        let hash = format!("{}", oid)[..7].to_string();
        let summary = commit.summary().unwrap_or("").to_string();
        let author = commit.author().name().unwrap_or("?").to_string();
        let time_secs = commit.time().seconds();
        let decor = decorations.get(&oid).cloned().unwrap_or_default();

        entries.push(CommitEntry {
            hash,
            summary,
            author,
            time_secs,
            decorations: decor,
        });
        if entries.len() >= limit {
            break;
        }
    }
    LogState::Ready(entries)
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

fn build_decoration_map(repo: &git2::Repository) -> std::collections::HashMap<git2::Oid, Vec<String>> {
    let mut map: std::collections::HashMap<git2::Oid, Vec<String>> = std::collections::HashMap::new();
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
        for delta in diff.deltas() {
            if let Some(p) = delta.new_file().path() {
                if p == path {
                    return true;
                }
            }
        }
    }
    false
}
