//! File finder completer — walks project files, fuzzy-matches against input.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use ignore::WalkBuilder;
use txv_core::complete::{Completer, CompletionVisitor};
use txv_widgets::fuzzy_match_positions;

use crate::completer_entry::Entry;

/// Completer that fuzzy-matches project file paths.
pub struct FileFinderCompleter {
    root: PathBuf,
    cache: Arc<Mutex<Vec<String>>>,
}

impl FileFinderCompleter {
    pub fn new(root: PathBuf) -> Self {
        let cache = Arc::new(Mutex::new(Vec::new()));
        let c = Self {
            root: root.clone(),
            cache: Arc::clone(&cache),
        };
        // Load file list in background
        thread::spawn(move || {
            let files = collect_files(&root);
            if let Ok(mut guard) = cache.lock() {
                *guard = files;
            }
        });
        c
    }
}

impl Completer for FileFinderCompleter {
    fn complete(
        &self,
        input: &str,
        _cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let needle = input.to_lowercase();
        if needle.is_empty() {
            return Ok(());
        }
        let Ok(guard) = self.cache.lock() else {
            return Ok(());
        };
        let mut scored: Vec<&String> = guard
            .iter()
            .filter(|path| fuzzy_match_positions(&path.to_lowercase(), &needle).is_some())
            .collect();
        scored.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
        for path in scored.into_iter().take(20) {
            let entry = Entry {
                text: path.clone(),
                display: path.clone(),
                kind: "file",
            };
            if !visitor(&entry)? {
                break;
            }
        }
        Ok(())
    }
}

fn collect_files(root: &Path) -> Vec<String> {
    let prefix_len = root.to_string_lossy().len() + 1; // +1 for trailing /
    let walker = WalkBuilder::new(root).hidden(true).build();
    let mut files = Vec::new();
    for entry in walker.flatten() {
        if entry.file_type().is_some_and(|ft| ft.is_file()) {
            let path_str = entry.path().to_string_lossy();
            if path_str.len() > prefix_len {
                files.push(path_str[prefix_len..].to_string());
            }
        }
    }
    files
}
