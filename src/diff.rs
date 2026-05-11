//! Unified diff generation using the `similar` crate.

use std::path::Path;

use similar::{ChangeTag, TextDiff};

/// Options for diff generation.
pub struct DiffOptions {
    /// Number of context lines around changes (default 3).
    pub context: usize,
    /// Ignore whitespace differences.
    pub ignore_whitespace: bool,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            context: 3,
            ignore_whitespace: false,
        }
    }
}

/// Generate a unified diff between `old` and `new` text.
/// Returns lines tagged for coloring: (tag, text) where tag is ' ', '+', '-', or '@'.
pub fn unified_diff(old: &str, new: &str, old_label: &str, new_label: &str, opts: &DiffOptions) -> Vec<(char, String)> {
    let (old_text, new_text) = if opts.ignore_whitespace {
        (normalize_whitespace(old), normalize_whitespace(new))
    } else {
        (old.to_string(), new.to_string())
    };

    let diff = TextDiff::from_lines(&old_text, &new_text);
    let mut result = Vec::new();

    result.push(('-', format!("--- {old_label}")));
    result.push(('+', format!("+++ {new_label}")));

    let mut udiff = diff.unified_diff();
    let hunks = udiff.context_radius(opts.context).iter_hunks();

    for hunk in hunks {
        result.push(('@', hunk.header().to_string()));
        for change in hunk.iter_changes() {
            let tag = match change.tag() {
                ChangeTag::Equal => ' ',
                ChangeTag::Insert => '+',
                ChangeTag::Delete => '-',
            };
            let line = change.to_string_lossy();
            let text = line.strip_suffix('\n').unwrap_or(&line);
            result.push((tag, text.to_string()));
        }
    }

    result
}

/// Read file content at a given revspec from git.
/// revspec examples: "HEAD", "main", "origin/main", "abc123", "HEAD~3"
pub fn git_file_content(root: &Path, rel_path: &str, revspec: &str) -> Result<String, String> {
    let repo = git2::Repository::discover(root).map_err(|e| format!("git: {e}"))?;
    let spec = format!("{revspec}:{rel_path}");
    let obj = repo
        .revparse_single(&spec)
        .map_err(|e| format!("revparse '{spec}': {e}"))?;
    let blob = obj.peel_to_blob().map_err(|e| format!("blob: {e}"))?;
    let content = std::str::from_utf8(blob.content()).map_err(|_| "binary file".to_string())?;
    Ok(content.to_string())
}

/// Normalize whitespace: collapse runs of whitespace to single space, trim trailing.
fn normalize_whitespace(text: &str) -> String {
    text.lines()
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_changes_produces_header_only() {
        let result = unified_diff("a\nb\n", "a\nb\n", "old", "new", &DiffOptions::default());
        // Only header lines, no hunks
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, '-');
        assert_eq!(result[1].0, '+');
    }

    #[test]
    fn added_line_shows_plus() {
        let result = unified_diff("a\n", "a\nb\n", "old", "new", &DiffOptions::default());
        let plus_lines: Vec<_> = result
            .iter()
            .filter(|(t, l)| *t == '+' && !l.starts_with("+++"))
            .collect();
        assert!(!plus_lines.is_empty());
        assert!(plus_lines.iter().any(|(_, l)| l == "b"));
    }

    #[test]
    fn removed_line_shows_minus() {
        let result = unified_diff("a\nb\n", "a\n", "old", "new", &DiffOptions::default());
        let minus_lines: Vec<_> = result
            .iter()
            .filter(|(t, l)| *t == '-' && !l.starts_with("---"))
            .collect();
        assert!(!minus_lines.is_empty());
        assert!(minus_lines.iter().any(|(_, l)| l == "b"));
    }

    #[test]
    fn context_controls_surrounding_lines() {
        let old = "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n";
        let new = "1\n2\n3\n4\nX\n6\n7\n8\n9\n10\n";
        let opts = DiffOptions {
            context: 1,
            ignore_whitespace: false,
        };
        let result = unified_diff(old, new, "a", "b", &opts);
        let context_lines: Vec<_> = result.iter().filter(|(t, _)| *t == ' ').collect();
        // With context=1, should have at most 2 context lines (1 before + 1 after)
        assert!(context_lines.len() <= 2);
    }

    #[test]
    fn ignore_whitespace_flag() {
        let old = "hello   world\n";
        let new = "hello world\n";
        let opts = DiffOptions {
            context: 3,
            ignore_whitespace: true,
        };
        let result = unified_diff(old, new, "a", "b", &opts);
        // No hunks — only header
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn git_file_content_nonexistent_repo() {
        let dir = tempfile::tempdir().unwrap();
        let result = git_file_content(dir.path(), "foo.txt", "HEAD");
        assert!(result.is_err());
    }

    #[test]
    fn git_file_content_from_commit() {
        let dir = tempfile::tempdir().unwrap();
        let repo = git2::Repository::init(dir.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test").unwrap();
        config.set_str("user.email", "t@t.com").unwrap();

        std::fs::write(dir.path().join("f.txt"), "original\n").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("f.txt")).unwrap();
        index.write().unwrap();
        let tree_oid = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_oid).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[]).unwrap();

        let content = git_file_content(dir.path(), "f.txt", "HEAD").unwrap();
        assert_eq!(content, "original\n");
    }
}
