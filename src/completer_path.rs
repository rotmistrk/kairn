//! File path completion logic.

use std::path::{Path, PathBuf};

use txv_core::complete::CompletionVisitor;

use super::Entry;

fn resolve_path_parts<'a>(partial: &'a str, root: &Path) -> (PathBuf, &'a str, String) {
    if partial.is_empty() {
        return (root.to_path_buf(), "", String::new());
    }
    if partial.ends_with('/') || partial.ends_with(std::path::MAIN_SEPARATOR) {
        return (root.join(partial), "", partial.to_string());
    }
    if partial.contains('/') || partial.contains(std::path::MAIN_SEPARATOR) {
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

    let Ok(entries) = std::fs::read_dir(&search_dir) else {
        return Ok(());
    };

    let mut results: Vec<Entry> = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if !name_str.starts_with(prefix) {
            continue;
        }
        let rel_path = format!("{dir_prefix}{name_str}");
        let is_dir = entry.path().is_dir();
        let display = if is_dir {
            format!("{name_str}/")
        } else {
            name_str.to_string()
        };
        results.push(Entry {
            text: format!("edit {rel_path}"),
            display,
            kind: if is_dir {
                "dir"
            } else {
                "file"
            },
        });
    }
    results.sort_by(|a, b| a.display.cmp(&b.display));
    for e in &results {
        if !visitor(e)? {
            break;
        }
    }
    Ok(())
}
