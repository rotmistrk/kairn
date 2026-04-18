// Git operations: diff vs HEAD, commit log.

use std::path::Path;

use anyhow::{Context, Result};
use gix::bstr::ByteSlice;
use similar::TextDiff;

// ── Diff types ───────────────────────────

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

// ── Diff vs HEAD ────────────────────────────────

pub fn diff_vs_head(file_path: &Path) -> Result<Option<Vec<DiffLine>>> {
    let head_content = match read_head_blob(file_path)? {
        Some(c) => c,
        None => return Ok(None),
    };
    let work_content = std::fs::read_to_string(file_path)
        .with_context(|| format!("reading {}", file_path.display()))?;
    Ok(Some(build_unified_diff(
        &head_content,
        &work_content,
        file_path,
    )))
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

// ── Git log ──────────────────────────────

#[derive(Debug)]
pub struct LogEntry {
    pub hash_short: String,
    pub author: String,
    pub date: String,
    pub message: String,
}

pub fn git_log(workspace: &Path, file_path: Option<&str>, limit: usize) -> Result<Vec<LogEntry>> {
    let repo: gix::Repository = gix::discover(workspace).context("discovering git repo")?;
    let head = repo.head_id().context("reading HEAD")?;
    let walk = head.ancestors().all().context("walking ancestors")?;
    let mut entries = Vec::new();

    for info in walk {
        if entries.len() >= limit {
            break;
        }
        let id = match info {
            Ok(i) => i.id,
            Err(_) => continue,
        };
        let commit = match repo.find_object(id) {
            Ok(o) => o.into_commit(),
            Err(_) => continue,
        };
        if let Some(e) = build_log_entry(&repo, &commit, file_path) {
            entries.push(e);
        }
    }
    Ok(entries)
}

fn build_log_entry(
    repo: &gix::Repository,
    commit: &gix::Commit<'_>,
    file_path: Option<&str>,
) -> Option<LogEntry> {
    let decoded = commit.decode().ok()?;

    if let Some(fp) = file_path {
        if !file_changed_in_commit(repo, commit, &decoded, fp) {
            return None;
        }
    }

    let hash_short = format!("{:.7}", commit.id);
    let author = decoded.author.name.to_string();
    let date = format_epoch(decoded.author.time.seconds, decoded.author.time.offset);
    let msg = decoded.message.to_str_lossy();
    let message = msg.lines().next().unwrap_or("").to_string();

    Some(LogEntry {
        hash_short,
        author,
        date,
        message,
    })
}

fn file_changed_in_commit(
    repo: &gix::Repository,
    commit: &gix::Commit<'_>,
    decoded: &gix::objs::CommitRef<'_>,
    rel_path: &str,
) -> bool {
    let cur = blob_oid(commit, rel_path);
    let parent = parent_blob_oid(repo, decoded, rel_path);
    cur != parent
}

fn blob_oid(commit: &gix::Commit<'_>, path: &str) -> Option<gix::ObjectId> {
    commit
        .tree()
        .ok()?
        .lookup_entry_by_path(path)
        .ok()
        .flatten()
        .map(|e| e.oid().to_owned())
}

fn parent_blob_oid(
    repo: &gix::Repository,
    decoded: &gix::objs::CommitRef<'_>,
    path: &str,
) -> Option<gix::ObjectId> {
    let pid = decoded.parents().next()?;
    let parent = repo.find_object(pid).ok()?.into_commit();
    let tree = parent.tree().ok()?;
    tree.lookup_entry_by_path(path)
        .ok()
        .flatten()
        .map(|e| e.oid().to_owned())
}

fn format_epoch(secs: gix::date::SecondsSinceUnixEpoch, offset: i32) -> String {
    let total = secs + offset as i64;
    let days = total / 86400;
    let rem = total % 86400;
    let (y, m, d) = civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{d:02} {:02}:{:02}",
        rem / 3600,
        (rem % 3600) / 60
    )
}

fn civil_from_days(mut days: i64) -> (i64, i64, i64) {
    days += 719468;
    let era = days / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}
