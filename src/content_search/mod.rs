//! Workspace-wide text/regex search using `ignore` + `regex` crates.
//!
//! Pure data module — no rendering. Returns [`SearchMatch`] results
//! that the panel layer can display.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// A single match in a file.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Path relative to workspace root.
    pub path: String,
    /// 1-based line number.
    pub line_no: usize,
    /// The full line text.
    pub line_text: String,
    /// Byte offset of match start within the line.
    pub col_start: usize,
    /// Byte offset of match end within the line.
    pub col_end: usize,
}

/// Result of a workspace search.
#[derive(Debug, Clone)]
pub struct SearchResults {
    /// The query pattern used.
    pub pattern: String,
    /// All matches found.
    pub matches: Vec<SearchMatch>,
    /// Number of files scanned.
    pub files_scanned: usize,
    /// Whether the search was truncated at the limit.
    pub truncated: bool,
}

/// Maximum matches before truncation.
const MAX_MATCHES: usize = 1000;

/// Search all files under `root` for lines matching `pattern`.
///
/// Respects `.gitignore` via the `ignore` crate. Uses `regex` for
/// pattern matching. Returns up to [`MAX_MATCHES`] results.
pub fn search(root: &Path, pattern: &str, case_sensitive: bool) -> Result<SearchResults> {
    let re = build_regex(pattern, case_sensitive)?;
    let mut results = SearchResults {
        pattern: pattern.to_string(),
        matches: Vec::new(),
        files_scanned: 0,
        truncated: false,
    };
    let walker = ignore::WalkBuilder::new(root).hidden(true).build();
    let root_prefix = root.to_string_lossy();
    for entry in walker.flatten() {
        if results.matches.len() >= MAX_MATCHES {
            results.truncated = true;
            break;
        }
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        search_file(entry.path(), &root_prefix, &re, &mut results);
    }
    Ok(results)
}

fn build_regex(pattern: &str, case_sensitive: bool) -> Result<regex::Regex> {
    regex::RegexBuilder::new(pattern)
        .case_insensitive(!case_sensitive)
        .build()
        .with_context(|| format!("invalid regex: {pattern}"))
}

fn search_file(path: &Path, root_prefix: &str, re: &regex::Regex, results: &mut SearchResults) {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return, // skip binary / unreadable files
    };
    results.files_scanned += 1;
    let rel = relative_path(path, root_prefix);
    for (idx, line) in content.lines().enumerate() {
        if results.matches.len() >= MAX_MATCHES {
            results.truncated = true;
            return;
        }
        if let Some(m) = re.find(line) {
            results.matches.push(SearchMatch {
                path: rel.clone(),
                line_no: idx + 1,
                line_text: line.to_string(),
                col_start: m.start(),
                col_end: m.end(),
            });
        }
    }
}

fn relative_path(path: &Path, root_prefix: &str) -> String {
    let full = path.to_string_lossy();
    full.strip_prefix(root_prefix)
        .unwrap_or(&full)
        .trim_start_matches('/')
        .to_string()
}

/// Search for a literal string (no regex interpretation).
pub fn search_literal(root: &Path, text: &str, case_sensitive: bool) -> Result<SearchResults> {
    search(root, &regex::escape(text), case_sensitive)
}

/// List files matching a glob pattern under `root`.
pub fn find_files(root: &Path, glob_pattern: &str) -> Result<Vec<PathBuf>> {
    let glob = build_glob_regex(glob_pattern)?;
    let walker = ignore::WalkBuilder::new(root).hidden(true).build();
    let mut files = Vec::new();
    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let name = entry
            .path()
            .file_name()
            .map(|n| n.to_string_lossy())
            .unwrap_or_default();
        if glob.is_match(&name) {
            files.push(entry.into_path());
        }
    }
    Ok(files)
}

fn build_glob_regex(pattern: &str) -> Result<regex::Regex> {
    let re_str = format!(
        "^{}$",
        pattern
            .replace('.', r"\.")
            .replace('*', ".*")
            .replace('?', ".")
    );
    regex::Regex::new(&re_str).with_context(|| format!("invalid glob: {pattern}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("hello.rs"),
            "fn main() {\n    println!(\"hello\");\n}\n",
        )
        .unwrap();
        fs::write(
            dir.path().join("lib.rs"),
            "pub fn greet() {\n    println!(\"hi\");\n}\n",
        )
        .unwrap();
        fs::create_dir_all(dir.path().join("sub")).unwrap();
        fs::write(
            dir.path().join("sub/deep.rs"),
            "// deep file\nfn deep() {}\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn search_finds_matches() {
        let dir = make_test_dir();
        let r = search(dir.path(), "println", false).unwrap();
        assert_eq!(r.matches.len(), 2);
        assert!(r.matches.iter().all(|m| m.line_text.contains("println")));
    }

    #[test]
    fn search_case_insensitive() {
        let dir = make_test_dir();
        let r = search(dir.path(), "PRINTLN", false).unwrap();
        assert_eq!(r.matches.len(), 2);
    }

    #[test]
    fn search_case_sensitive() {
        let dir = make_test_dir();
        let r = search(dir.path(), "PRINTLN", true).unwrap();
        assert!(r.matches.is_empty());
    }

    #[test]
    fn search_literal_escapes_regex() {
        let dir = make_test_dir();
        let r = search_literal(dir.path(), "fn main()", false).unwrap();
        assert_eq!(r.matches.len(), 1);
    }

    #[test]
    fn search_no_matches() {
        let dir = make_test_dir();
        let r = search(dir.path(), "zzzznonexistent", false).unwrap();
        assert!(r.matches.is_empty());
    }

    #[test]
    fn search_reports_line_numbers() {
        let dir = make_test_dir();
        let r = search(dir.path(), "println", false).unwrap();
        assert!(r.matches.iter().all(|m| m.line_no > 0));
    }

    #[test]
    fn find_files_by_glob() {
        let dir = make_test_dir();
        let files = find_files(dir.path(), "*.rs").unwrap();
        assert!(files.len() >= 3);
    }

    #[test]
    fn invalid_regex_returns_error() {
        let dir = make_test_dir();
        assert!(search(dir.path(), "[invalid", false).is_err());
    }
}
