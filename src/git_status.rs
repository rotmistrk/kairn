//! Git status collection — runs `git status` and parses output.

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

/// File status from git.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Clean,
    Modified,
    Added,
    Untracked,
    Ignored,
}

/// Collect git status for all files under `root`.
/// Returns relative path → status. Returns empty map on error.
pub fn collect_git_status(root: &Path) -> HashMap<String, FileStatus> {
    let output = Command::new("git")
        .args([
            "-C",
            &root.display().to_string(),
            "status",
            "--porcelain=v1",
            "-z",
            "--ignored",
        ])
        .output();

    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return HashMap::new(),
    };

    parse_porcelain(&output.stdout)
}

/// Parse `git status --porcelain=v1 -z` output into a status map.
pub fn parse_porcelain(data: &[u8]) -> HashMap<String, FileStatus> {
    let mut map = HashMap::new();
    let text = String::from_utf8_lossy(data);
    let entries: Vec<&str> = text.split('\0').collect();

    let mut i = 0;
    while i < entries.len() {
        let entry = entries[i];
        if entry.len() < 4 {
            i += 1;
            continue;
        }
        let x = entry.as_bytes()[0];
        let y = entry.as_bytes()[1];
        let path = &entry[3..];

        let status = match (x, y) {
            (b'?', b'?') => FileStatus::Untracked,
            (b'!', b'!') => FileStatus::Ignored,
            (b'A', _) => FileStatus::Added,
            (_, b'M') | (b'M', _) => FileStatus::Modified,
            (_, b'D') | (b'D', _) => FileStatus::Modified,
            (b'R', _) => {
                // Rename: skip the "from" path (next NUL-separated entry)
                i += 1;
                FileStatus::Modified
            }
            _ => FileStatus::Clean,
        };

        if !path.is_empty() {
            map.insert(path.to_string(), status);
        }
        i += 1;
    }
    map
}

/// Determine the "most important" status for a directory from its children.
/// Priority: Untracked > Modified > Added > Clean.
pub fn dir_status(children: &[FileStatus]) -> FileStatus {
    let mut result = FileStatus::Clean;
    for &s in children {
        let priority = status_priority(s);
        if priority > status_priority(result) {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_untracked() {
        let data = b"?? new.rs\0";
        let map = parse_porcelain(data);
        assert_eq!(map.get("new.rs"), Some(&FileStatus::Untracked));
    }

    #[test]
    fn parse_modified() {
        let data = b" M changed.rs\0";
        let map = parse_porcelain(data);
        assert_eq!(map.get("changed.rs"), Some(&FileStatus::Modified));
    }

    #[test]
    fn parse_added() {
        let data = b"A  staged.rs\0";
        let map = parse_porcelain(data);
        assert_eq!(map.get("staged.rs"), Some(&FileStatus::Added));
    }

    #[test]
    fn parse_ignored() {
        let data = b"!! target/debug\0";
        let map = parse_porcelain(data);
        assert_eq!(map.get("target/debug"), Some(&FileStatus::Ignored));
    }

    #[test]
    fn parse_multiple() {
        let data = b"?? new.rs\0 M lib.rs\0A  added.rs\0";
        let map = parse_porcelain(data);
        assert_eq!(map.len(), 3);
        assert_eq!(map.get("new.rs"), Some(&FileStatus::Untracked));
        assert_eq!(map.get("lib.rs"), Some(&FileStatus::Modified));
        assert_eq!(map.get("added.rs"), Some(&FileStatus::Added));
    }

    #[test]
    fn parse_empty() {
        let map = parse_porcelain(b"");
        assert!(map.is_empty());
    }

    #[test]
    fn parse_rename_skips_from_path() {
        let data = b"R  new_name.rs\0old_name.rs\0";
        let map = parse_porcelain(data);
        assert_eq!(map.get("new_name.rs"), Some(&FileStatus::Modified));
        assert!(!map.contains_key("old_name.rs"));
    }

    #[test]
    fn dir_status_picks_highest_priority() {
        assert_eq!(
            dir_status(&[FileStatus::Clean, FileStatus::Modified]),
            FileStatus::Modified
        );
        assert_eq!(
            dir_status(&[FileStatus::Added, FileStatus::Untracked]),
            FileStatus::Untracked
        );
        assert_eq!(dir_status(&[FileStatus::Clean, FileStatus::Ignored]), FileStatus::Clean);
    }

    #[test]
    fn dir_status_empty_is_clean() {
        assert_eq!(dir_status(&[]), FileStatus::Clean);
    }
}
