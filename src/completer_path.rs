//! File path completion logic.

use std::fs;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

use txv_core::complete::CompletionVisitor;

use super::Entry;

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

pub fn complete_path(
    partial: &str,
    root: &Path,
    visitor: &mut CompletionVisitor<'_>,
) -> Result<(), Box<dyn std::error::Error>> {
    let (search_dir, prefix, dir_prefix) = resolve_path_parts(partial, root);

    let Ok(entries) = fs::read_dir(&search_dir) else {
        return Ok(());
    };

    let mut results = collect_entries(entries, prefix, &dir_prefix);
    results.sort_by(|a, b| a.display.cmp(&b.display));

    // If single match is a directory, also list its contents.
    if results.len() == 1 && results[0].kind == "dir" {
        let sub_path = format!("{}/", results[0].text.strip_prefix("edit ").unwrap_or(""));
        let sub_dir = root.join(&sub_path);
        if let Ok(sub_entries) = fs::read_dir(&sub_dir) {
            let mut sub = collect_entries(sub_entries, "", &sub_path);
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

fn collect_entries(entries: fs::ReadDir, prefix: &str, dir_prefix: &str) -> Vec<Entry> {
    let mut results: Vec<Entry> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        let rel_path = format!("{dir_prefix}{name_str}");
        let is_dir = entry.path().is_dir();
        let (text, display) = if is_dir {
            (format!("edit {rel_path}/"), format!("{name_str}/"))
        } else {
            (format!("edit {rel_path}"), name_str.to_string())
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
