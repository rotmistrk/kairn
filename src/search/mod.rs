// Incremental fuzzy search across workspace files.

use std::path::Path;

use crossterm::event::{KeyCode, KeyEvent};
use nucleo::pattern::{CaseMatching, Normalization, Pattern};
use nucleo::{Matcher, Utf32Str};

/// Collected file paths and fuzzy matcher state.
pub struct FileSearch {
    /// All file paths relative to workspace root.
    paths: Vec<String>,
    /// Current query string.
    pub query: String,
    /// Cursor position in query.
    pub cursor: usize,
    /// Matched results (indices into paths, best first).
    pub results: Vec<SearchResult>,
    /// Selected result index.
    pub selected: usize,
}

pub struct SearchResult {
    pub path_index: usize,
    pub score: u32,
}

impl FileSearch {
    /// Scan workspace and collect all file paths.
    pub fn new(root: &Path) -> Self {
        let paths = collect_paths(root);
        Self {
            paths,
            query: String::new(),
            cursor: 0,
            results: Vec::new(),
            selected: 0,
        }
    }

    /// Re-run the fuzzy match against current query.
    pub fn update_results(&mut self) {
        self.results.clear();
        if self.query.is_empty() {
            // Show all files when query is empty (capped)
            self.results = self
                .paths
                .iter()
                .enumerate()
                .take(100)
                .map(|(i, _)| SearchResult {
                    path_index: i,
                    score: 0,
                })
                .collect();
        } else {
            let pattern = Pattern::new(
                &self.query,
                CaseMatching::Smart,
                Normalization::Smart,
                nucleo::pattern::AtomKind::Fuzzy,
            );
            let mut buf = Vec::new();
            let mut matcher = Matcher::default();
            let mut scored: Vec<(usize, u32)> = self
                .paths
                .iter()
                .enumerate()
                .filter_map(|(i, path)| {
                    buf.clear();
                    let haystack = Utf32Str::new(path, &mut buf);
                    pattern.score(haystack, &mut matcher).map(|s| (i, s))
                })
                .collect();
            scored.sort_by(|a, b| b.1.cmp(&a.1));
            scored.truncate(100);
            self.results = scored
                .into_iter()
                .map(|(path_index, score)| SearchResult { path_index, score })
                .collect();
        }
        self.selected = 0;
    }

    /// Get the display path for a result.
    pub fn result_path(&self, result: &SearchResult) -> &str {
        &self.paths[result.path_index]
    }

    /// Get the selected file path, if any.
    pub fn selected_path(&self) -> Option<&str> {
        self.results
            .get(self.selected)
            .map(|r| self.paths[r.path_index].as_str())
    }

    /// Handle a key event. Returns the action to take.
    pub fn handle_key(&mut self, key: KeyEvent) -> SearchAction {
        match key.code {
            KeyCode::Esc => SearchAction::Close,
            KeyCode::Enter => self.accept_selected(),
            KeyCode::Up => {
                self.move_selection(-1);
                SearchAction::None
            }
            KeyCode::Down => {
                self.move_selection(1);
                SearchAction::None
            }
            KeyCode::Backspace => {
                self.delete_char();
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

    fn accept_selected(&self) -> SearchAction {
        match self.selected_path() {
            Some(p) => SearchAction::Open(p.to_string()),
            None => SearchAction::Close,
        }
    }

    fn move_selection(&mut self, delta: isize) {
        let new = (self.selected as isize) + delta;
        let max = self.results.len().saturating_sub(1) as isize;
        self.selected = new.clamp(0, max) as usize;
    }

    fn delete_char(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.query.remove(self.cursor);
            self.update_results();
        }
    }
}

pub enum SearchAction {
    None,
    Open(String),
    Close,
}

fn collect_paths(root: &Path) -> Vec<String> {
    let mut paths = Vec::new();
    let walker = ignore::WalkBuilder::new(root).hidden(true).build();
    let root_str = root.to_string_lossy();
    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            let full = entry.path().to_string_lossy();
            // Store relative path
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
