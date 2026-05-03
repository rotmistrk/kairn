//! Fuzzy file search across workspace files via `nucleo`.
//!
//! Pure data model — no key handling or rendering. The panel layer
//! drives the search by calling [`FileSearch::set_query`] and reading
//! [`FileSearch::selected_path`].

use std::path::Path;

use nucleo::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo::{Matcher, Utf32Str};

/// A single fuzzy match result.
pub struct SearchResult {
    /// Index into the internal path list.
    pub path_index: usize,
    /// Match score (higher is better).
    pub score: u32,
}

/// Fuzzy file search state.
pub struct FileSearch {
    paths: Vec<String>,
    /// Current query string.
    pub query: String,
    /// Cursor position within the query (byte offset).
    pub cursor: usize,
    /// Matched results (best first).
    pub results: Vec<SearchResult>,
    /// Index of the currently selected result.
    pub selected: usize,
}

/// Action returned by search key handling (used by panel layer).
pub enum SearchAction {
    /// No action needed.
    None,
    /// Open the file at this path.
    Open(String),
    /// Close the search overlay.
    Close,
}

/// Maximum number of results to return.
const MAX_RESULTS: usize = 100;

impl FileSearch {
    /// Scan `root` for files and create a new search instance.
    pub fn new(root: &Path) -> Self {
        Self {
            paths: collect_paths(root),
            query: String::new(),
            cursor: 0,
            results: Vec::new(),
            selected: 0,
        }
    }

    /// Update the query and recompute results.
    pub fn set_query(&mut self, query: &str) {
        self.query = query.to_string();
        self.update_results();
    }

    /// Re-run the fuzzy match against the current query.
    pub fn update_results(&mut self) {
        self.results.clear();
        if self.query.is_empty() {
            self.results = self
                .paths
                .iter()
                .enumerate()
                .take(MAX_RESULTS)
                .map(|(i, _)| SearchResult {
                    path_index: i,
                    score: 0,
                })
                .collect();
        } else {
            self.results = score_paths(&self.paths, &self.query);
        }
        self.selected = 0;
    }

    /// Get the display path for a result.
    pub fn result_path(&self, result: &SearchResult) -> &str {
        &self.paths[result.path_index]
    }

    /// Get the currently selected file path, if any.
    pub fn selected_path(&self) -> Option<&str> {
        self.results
            .get(self.selected)
            .map(|r| self.paths[r.path_index].as_str())
    }

    /// Move selection by `delta` (positive = down, negative = up).
    pub fn move_selection(&mut self, delta: isize) {
        let max = self.results.len().saturating_sub(1) as isize;
        let new = (self.selected as isize + delta).clamp(0, max);
        self.selected = new as usize;
    }

    /// Total number of collected paths.
    pub fn total_files(&self) -> usize {
        self.paths.len()
    }

    /// Handle a crossterm key event (backward-compatible shim).
    pub fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> SearchAction {
        use crossterm::event::KeyCode;
        match key.code {
            KeyCode::Esc => SearchAction::Close,
            KeyCode::Enter => match self.selected_path() {
                Some(p) => SearchAction::Open(p.to_string()),
                None => SearchAction::Close,
            },
            KeyCode::Up => {
                self.move_selection(-1);
                SearchAction::None
            }
            KeyCode::Down => {
                self.move_selection(1);
                SearchAction::None
            }
            KeyCode::Backspace => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                    self.query.remove(self.cursor);
                    self.update_results();
                }
                SearchAction::None
            }
            KeyCode::Left => {
                self.cursor = self.cursor.saturating_sub(1);
                SearchAction::None
            }
            KeyCode::Right => {
                if self.cursor < self.query.len() {
                    self.cursor += 1;
                }
                SearchAction::None
            }
            KeyCode::Char(c) => {
                self.query.insert(self.cursor, c);
                self.cursor += c.len_utf8();
                self.update_results();
                SearchAction::None
            }
            _ => SearchAction::None,
        }
    }
}

fn score_paths(paths: &[String], query: &str) -> Vec<SearchResult> {
    let pattern = Pattern::new(
        query,
        CaseMatching::Smart,
        Normalization::Smart,
        AtomKind::Fuzzy,
    );
    let mut buf = Vec::new();
    let mut matcher = Matcher::default();
    let mut scored: Vec<(usize, u32)> = paths
        .iter()
        .enumerate()
        .filter_map(|(i, path)| {
            buf.clear();
            let haystack = Utf32Str::new(path, &mut buf);
            pattern.score(haystack, &mut matcher).map(|s| (i, s))
        })
        .collect();
    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(MAX_RESULTS);
    scored
        .into_iter()
        .map(|(path_index, score)| SearchResult { path_index, score })
        .collect()
}

fn collect_paths(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    let walker = ignore::WalkBuilder::new(root).hidden(true).build();
    let root_str = root.to_string_lossy();
    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            let full = entry.path().to_string_lossy();
            let rel = full
                .strip_prefix(root_str.as_ref())
                .unwrap_or(&full)
                .trim_start_matches('/');
            if !rel.is_empty() {
                paths.push(rel.to_string());
            }
        }
    }
    paths.sort();
    paths
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.rs"), "").unwrap();
        fs::write(dir.path().join("lib.rs"), "").unwrap();
        fs::create_dir_all(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/app.rs"), "").unwrap();
        dir
    }

    #[test]
    fn collects_files() {
        let dir = make_test_dir();
        let search = FileSearch::new(dir.path());
        assert!(search.total_files() >= 3);
    }

    #[test]
    fn empty_query_returns_all() {
        let dir = make_test_dir();
        let mut search = FileSearch::new(dir.path());
        search.set_query("");
        assert!(!search.results.is_empty());
    }

    #[test]
    fn fuzzy_match_filters() {
        let dir = make_test_dir();
        let mut search = FileSearch::new(dir.path());
        search.set_query("main");
        assert!(search
            .results
            .iter()
            .any(|r| { search.result_path(r).contains("main") }));
    }

    #[test]
    fn move_selection_clamps() {
        let dir = make_test_dir();
        let mut search = FileSearch::new(dir.path());
        search.set_query("");
        search.move_selection(-100);
        assert_eq!(search.selected, 0);
        search.move_selection(10000);
        assert!(search.selected < search.results.len());
    }

    #[test]
    fn selected_path_returns_none_when_empty() {
        let dir = make_test_dir();
        let mut search = FileSearch::new(dir.path());
        search.set_query("zzzznonexistent");
        assert!(search.selected_path().is_none());
    }
}
