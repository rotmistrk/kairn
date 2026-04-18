// Git diff: read HEAD version via gix, compute unified diff via similar.

use std::path::Path;

use anyhow::{Context, Result};
use similar::TextDiff;

/// A single line in a unified diff.
#[derive(Debug)]
pub struct DiffLine {
    pub tag: DiffTag,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Header,
    Context,
    Added,
    Removed,
}

/// Compute a unified diff between the HEAD version and the working copy.
pub fn diff_vs_head(file_path: &Path) -> Result<Option<Vec<DiffLine>>> {
    let head_content = match read_head_blob(file_path)? {
        Some(c) => c,
        None => return Ok(None),
    };
    let work_content = std::fs::read_to_string(file_path)
        .with_context(|| format!("reading {}", file_path.display()))?;

    let lines = build_unified_diff(&head_content, &work_content, file_path);
    Ok(Some(lines))
}

fn read_head_blob(file_path: &Path) -> Result<Option<String>> {
    let abs = std::fs::canonicalize(file_path)
        .with_context(|| format!("canonicalize {}", file_path.display()))?;

    let repo: gix::Repository = match gix::discover(&abs) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let workdir = match repo.work_dir() {
        Some(w) => w.to_path_buf(),
        None => return Ok(None),
    };

    let rel = match abs.strip_prefix(&workdir) {
        Ok(r) => r,
        Err(_) => return Ok(None),
    };

    let head_id = match repo.head_id() {
        Ok(id) => id,
        Err(_) => return Ok(None),
    };

    let commit = head_id.object()?.into_commit();
    let tree = commit.tree().context("reading HEAD tree")?;
    let rel_str = rel.to_string_lossy();
    let entry = match tree.lookup_entry_by_path(rel_str.as_ref()) {
        Ok(Some(e)) => e,
        _ => return Ok(None),
    };

    let object = entry.object().context("reading blob")?;
    Ok(Some(String::from_utf8_lossy(&object.data).to_string()))
}

fn build_unified_diff(old: &str, new: &str, path: &Path) -> Vec<DiffLine> {
    let mut lines = Vec::new();
    let name = path.to_string_lossy();

    lines.push(DiffLine {
        tag: DiffTag::Header,
        content: format!("--- a/{name} (HEAD)"),
    });
    lines.push(DiffLine {
        tag: DiffTag::Header,
        content: format!("+++ b/{name} (working)"),
    });

    let diff = TextDiff::from_lines(old, new);
    let udiff = diff.unified_diff().context_radius(3).to_string();
    append_udiff_lines(&mut lines, &udiff);

    if lines.len() <= 2 {
        lines.push(DiffLine {
            tag: DiffTag::Context,
            content: "(no changes)".to_string(),
        });
    }

    lines
}

fn append_udiff_lines(lines: &mut Vec<DiffLine>, udiff: &str) {
    // Skip the first two lines (--- / +++ from similar) since we add our own
    for line in udiff.lines().skip(2) {
        let (tag, content) = if let Some(rest) = line.strip_prefix("@@") {
            (DiffTag::Header, format!("@@{rest}"))
        } else if let Some(rest) = line.strip_prefix('+') {
            (DiffTag::Added, format!("+{rest}"))
        } else if let Some(rest) = line.strip_prefix('-') {
            (DiffTag::Removed, format!("-{rest}"))
        } else {
            (DiffTag::Context, line.to_string())
        };
        lines.push(DiffLine { tag, content });
    }
}
