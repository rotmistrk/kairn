//! Git status collection — uses git2 (libgit2), no subprocess.

use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FileStatus {
    Clean,
    Modified,
    Added,
    Untracked,
    Ignored,
    Conflict,
}

/// Collect git status for all files under `root` using libgit2.
/// Returns empty map if not a git repo or on any error.
pub fn collect_git_status(root: &Path) -> HashMap<String, FileStatus> {
    let repo = match git2::Repository::discover(root) {
        Ok(r) => r,
        Err(_) => return HashMap::new(),
    };

    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(true)
        .include_ignored(false);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();
    for entry in statuses.iter() {
        let Some(path) = entry.path() else {
            continue;
        };
        let s = entry.status();
        let status = if s.contains(git2::Status::CONFLICTED) {
            FileStatus::Conflict
        } else if s.intersects(git2::Status::WT_NEW | git2::Status::INDEX_NEW) {
            if s.contains(git2::Status::INDEX_NEW) {
                FileStatus::Added
            } else {
                FileStatus::Untracked
            }
        } else if s.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::INDEX_MODIFIED
                | git2::Status::WT_RENAMED
                | git2::Status::INDEX_RENAMED
                | git2::Status::WT_DELETED
                | git2::Status::INDEX_DELETED,
        ) {
            FileStatus::Modified
        } else if s.contains(git2::Status::IGNORED) {
            FileStatus::Ignored
        } else {
            continue;
        };
        map.insert(path.to_string(), status);
    }
    map
}

/// Determine the aggregate status for a directory from its children.
pub fn dir_status(children: &[FileStatus]) -> FileStatus {
    let mut result = FileStatus::Clean;
    for &s in children {
        if status_priority(s) > status_priority(result) {
            result = s;
        }
    }
    result
}

fn status_priority(s: FileStatus) -> u8 {
    match s {
        FileStatus::Clean => 0,
        FileStatus::Ignored => 0,
        FileStatus::Added => 1,
        FileStatus::Modified => 2,
        FileStatus::Untracked => 3,
        FileStatus::Conflict => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dir_status_picks_highest() {
        assert_eq!(
            dir_status(&[FileStatus::Clean, FileStatus::Modified]),
            FileStatus::Modified
        );
        assert_eq!(
            dir_status(&[FileStatus::Added, FileStatus::Untracked]),
            FileStatus::Untracked
        );
        assert_eq!(dir_status(&[]), FileStatus::Clean);
    }
}
