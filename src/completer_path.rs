//! Filesystem path completion with configurable entry filter.

use std::fs::{self, DirEntry};
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use txv_core::complete::CompletionVisitor;

use super::Entry;

/// Filter that accepts all entries (files + directories).
pub fn accept_all(_entry: &DirEntry) -> bool {
    true
}

/// Filter that accepts only directories.
pub fn accept_dirs(entry: &DirEntry) -> bool {
    entry.path().is_dir()
}

/// Complete filesystem paths, filtered by `accept`.
///
/// - `partial`: what the user typed after the command
/// - `root`: base directory for relative paths
/// - `cmd`: command name (used to build the full completion text)
/// - `accept`: predicate — only entries where `accept(&entry)` returns true are shown
/// - `visitor`: receives each completion candidate
pub fn complete_fs(
    partial: &str,
    root: &Path,
    cmd: &str,
    accept: &dyn Fn(&DirEntry) -> bool,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, prefix, dir_prefix) = resolve_path_parts(partial, root);

    let Ok(entries) = fs::read_dir(&search_dir) else {
        return Ok(());
    };

    let mut results = collect_entries(entries, prefix, &dir_prefix, cmd, accept);
    results.sort_by(|a, b| a.display.cmp(&b.display));

    // If single match is a directory, also list its contents.
    if results.len() == 1 && results[0].kind == "dir" {
        let cmd_prefix = format!("{cmd} ");
        let sub_path = results[0].text.strip_prefix(&cmd_prefix).unwrap_or("");
        let sub_dir = root.join(sub_path);
        if let Ok(sub_entries) = fs::read_dir(&sub_dir) {
            let mut sub = collect_entries(sub_entries, "", sub_path, cmd, accept);
            sub.sort_by(|a, b| a.display.cmp(&b.display));
            results.extend(sub);
        }
    }

    for e in &results {
        if !visitor(e)? {
            break;
        }
    }
    Ok(())
}

fn resolve_path_parts<'a>(partial: &'a str, root: &Path) -> (PathBuf, &'a str, String) {
    if partial.is_empty() {
        return (root.to_path_buf(), "", String::new());
    }
    if partial.ends_with('/') || partial.ends_with(MAIN_SEPARATOR) {
        return (root.join(partial), "", partial.to_string());
    }
    if partial.contains('/') || partial.contains(MAIN_SEPARATOR) {
        let p = Path::new(partial);
        let parent = p.parent().map(|d| d.to_str().unwrap_or(".")).unwrap_or(".");
        let prefix = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let dir_prefix = format!("{}/", parent);
        return (root.join(parent), prefix, dir_prefix);
    }
    (root.to_path_buf(), partial, String::new())
}

fn collect_entries(
    entries: fs::ReadDir,
    prefix: &str,
    dir_prefix: &str,
    cmd: &str,
    accept: &dyn Fn(&DirEntry) -> bool,
) -> Vec<Entry> {
    let mut results: Vec<Entry> = Vec::new();
    for entry in entries.flatten() {
        if !accept(&entry) {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        let is_dir = entry.path().is_dir();
        let rel_path = format!("{dir_prefix}{name_str}");
        let (text, display) = if is_dir {
            (format!("{cmd} {rel_path}/"), format!("{name_str}/"))
        } else {
            (format!("{cmd} {rel_path}"), name_str.to_string())
        };
        results.push(Entry {
            text,
            display,
            kind: if is_dir {
                "dir"
            } else {
                "file"
            },
        });
    }
    results
}
