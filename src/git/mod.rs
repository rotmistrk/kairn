//! Git operations: diff vs HEAD, commit log, blame.
//!
//! Pure data module — no rendering code. All functions return
//! plain data types suitable for consumption by any UI layer.

use std::path::Path;

use anyhow::{Context, Result};
use similar::TextDiff;

// ── Diff types ───────────────────────────

/// A single line in a unified diff.
#[derive(Debug, Clone)]
pub struct DiffLine {
    /// What kind of diff line this is.
    pub tag: DiffTag,
    /// The text content of the line.
    pub content: String,
}

/// Classification of a diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    /// File header (--- / +++ / @@).
    Header,
    /// Unchanged context line.
    Context,
    /// Added line.
    Added,
    /// Removed line.
    Removed,
}

// ── Diff vs HEAD ────────────────────────────────

/// Compute a unified diff of `file_path` against its HEAD version.
///
/// Returns `None` if the file is not tracked or no repo is found.
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
    let repo = match gix::discover(&abs) {
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
    let name = path.to_string_lossy();
    let mut lines = vec![
        DiffLine {
            tag: DiffTag::Header,
            content: format!("--- a/{name} (HEAD)"),
        },
        DiffLine {
            tag: DiffTag::Header,
            content: format!("+++ b/{name} (working)"),
        },
    ];
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
        let (tag, content) = classify_udiff_line(line);
        lines.push(DiffLine { tag, content });
    }
}

fn classify_udiff_line(line: &str) -> (DiffTag, String) {
    if let Some(rest) = line.strip_prefix("@@") {
        (DiffTag::Header, format!("@@{rest}"))
    } else if let Some(rest) = line.strip_prefix('+') {
        (DiffTag::Added, format!("+{rest}"))
    } else if let Some(rest) = line.strip_prefix('-') {
        (DiffTag::Removed, format!("-{rest}"))
    } else {
        (DiffTag::Context, line.to_string())
    }
}

// ── Git log ──────────────────────────────

/// A single commit in the log.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Short hash (7 chars).
    pub hash_short: String,
    /// Author name.
    pub author: String,
    /// Formatted date string.
    pub date: String,
    /// First line of commit message.
    pub message: String,
}

/// Retrieve commit log for the repo at `workspace`.
///
/// If `file_path` is provided, only commits touching that file are returned.
pub fn git_log(workspace: &Path, file_path: Option<&str>, limit: usize) -> Result<Vec<LogEntry>> {
    let repo = gix::discover(workspace).context("discovering git repo")?;
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
    use gix::bstr::ByteSlice;
    let decoded = commit.decode().ok()?;
    if let Some(fp) = file_path {
        if !file_changed_in_commit(repo, commit, &decoded, fp) {
            return None;
        }
    }
    let hash_short: String = format!("{}", commit.id).chars().take(7).collect();
    let author = decoded.author.name.to_string();
    let date = format_epoch(decoded.author.time.seconds, decoded.author.time.offset);
    let msg = decoded.message.to_str_lossy();
    let message = msg.lines().next().unwrap_or_default().to_string();
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
    parent
        .tree()
        .ok()?
        .lookup_entry_by_path(path)
        .ok()
        .flatten()
        .map(|e| e.oid().to_owned())
}

// ── Date formatting ──────────────────────

/// Format a Unix epoch timestamp with offset into `YYYY-MM-DD HH:MM`.
pub fn format_epoch(secs: gix::date::SecondsSinceUnixEpoch, offset: i32) -> String {
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
    days += 719_468;
    let era = days / 146_097;
    let doe = days - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

// ── Git blame ────────────────────────────────

/// A single line of blame output.
#[derive(Debug, Clone)]
pub struct BlameLine {
    /// Short commit hash.
    pub hash_short: String,
    /// Author name.
    pub author: String,
    /// Formatted date.
    pub date: String,
    /// 1-based line number in the file.
    pub line_no: usize,
    /// Line content.
    pub content: String,
}

/// Run `git blame --porcelain` on a file and parse the output.
pub fn git_blame(file_path: &Path) -> Result<Vec<BlameLine>> {
    let output = std::process::Command::new("git")
        .args(["blame", "--porcelain"])
        .arg(file_path)
        .output()
        .context("running git blame")?;
    if !output.status.success() {
        anyhow::bail!("git blame failed");
    }
    parse_porcelain_blame(&String::from_utf8_lossy(&output.stdout))
}

fn parse_porcelain_blame(text: &str) -> Result<Vec<BlameLine>> {
    let mut lines = Vec::new();
    let mut hash = String::new();
    let mut author = String::new();
    let mut date = String::new();
    let mut line_no = 0usize;

    for raw in text.lines() {
        if let Some(content) = raw.strip_prefix('\t') {
            lines.push(BlameLine {
                hash_short: hash.chars().take(7).collect(),
                author: author.clone(),
                date: date.clone(),
                line_no,
                content: content.to_string(),
            });
        } else if is_blame_header(raw) {
            let parts: Vec<&str> = raw.split_whitespace().collect();
            hash = parts.first().unwrap_or(&"").to_string();
            line_no = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
        } else if let Some(rest) = raw.strip_prefix("author ") {
            author = rest.to_string();
        } else if let Some(rest) = raw.strip_prefix("author-time ") {
            let secs: i64 = rest.parse().unwrap_or(0);
            date = format_epoch(secs, 0);
        }
    }
    Ok(lines)
}

fn is_blame_header(line: &str) -> bool {
    line.len() >= 40 && line.as_bytes()[0].is_ascii_hexdigit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn civil_from_days_epoch() {
        let (y, m, d) = civil_from_days(0);
        assert_eq!((y, m, d), (1970, 1, 1));
    }

    #[test]
    fn format_epoch_known_date() {
        // 2024-01-15 12:00 UTC
        let s = format_epoch(1_705_320_000, 0);
        assert!(s.starts_with("2024-01-15"));
    }

    #[test]
    fn classify_udiff_added() {
        let (tag, _) = classify_udiff_line("+new line");
        assert_eq!(tag, DiffTag::Added);
    }

    #[test]
    fn classify_udiff_removed() {
        let (tag, _) = classify_udiff_line("-old line");
        assert_eq!(tag, DiffTag::Removed);
    }

    #[test]
    fn classify_udiff_header() {
        let (tag, _) = classify_udiff_line("@@ -1,3 +1,4 @@");
        assert_eq!(tag, DiffTag::Header);
    }

    #[test]
    fn classify_udiff_context() {
        let (tag, _) = classify_udiff_line(" unchanged");
        assert_eq!(tag, DiffTag::Context);
    }

    #[test]
    fn parse_blame_empty() {
        let result = parse_porcelain_blame("").unwrap();
        assert!(result.is_empty());
    }
}
