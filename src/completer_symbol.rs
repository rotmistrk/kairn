//! Symbol finder completer — grep-based symbol search across project files.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use txv_core::complete::{Completer, CompletionVisitor};

use crate::completer_entry::Entry;

/// Completer that searches for symbol-like patterns in project files.
pub struct SymbolFinderCompleter {
    root: PathBuf,
    results: Arc<Mutex<(String, Vec<String>)>>,
}

impl SymbolFinderCompleter {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            results: Arc::new(Mutex::new((String::new(), Vec::new()))),
        }
    }
}

impl Completer for SymbolFinderCompleter {
    fn complete(
        &self,
        input: &str,
        _cursor: usize,
        visitor: &mut CompletionVisitor<'_>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let query = input.trim().to_string();
        if query.len() < 2 {
            return Ok(());
        }
        // Check if we already have results for this query
        {
            let guard = self.results.lock().map_err(|e| e.to_string())?;
            if guard.0 == query {
                for item in guard.1.iter().take(20) {
                    let entry = Entry {
                        text: item.clone(),
                        display: item.clone(),
                        kind: "symbol",
                    };
                    if !visitor(&entry)? {
                        break;
                    }
                }
                return Ok(());
            }
        }
        // Fire background search
        let root = self.root.clone();
        let slot = Arc::clone(&self.results);
        let q = query.clone();
        thread::spawn(move || {
            let results = grep_symbols(&root, &q);
            if let Ok(mut guard) = slot.lock() {
                *guard = (q, results);
            }
        });
        Ok(())
    }
}

fn grep_symbols(root: &Path, query: &str) -> Vec<String> {
    use ignore::WalkBuilder;

    let pattern = query.to_lowercase();
    let mut results = Vec::new();
    let walker = WalkBuilder::new(root).hidden(true).build();

    for entry in walker.flatten() {
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !is_code_file(ext) {
            continue;
        }
        let rel = path.strip_prefix(root).unwrap_or(path);
        scan_file_symbols(path, rel, ext, &pattern, &mut results);
        if results.len() >= 50 {
            break;
        }
    }
    results
}

fn scan_file_symbols(path: &Path, rel: &Path, ext: &str, pattern: &str, results: &mut Vec<String>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    for (line_no, line) in content.lines().enumerate() {
        if looks_like_definition(line, ext) && line.to_lowercase().contains(pattern) {
            results.push(format!("{}:{} {}", rel.display(), line_no + 1, line.trim()));
            if results.len() >= 50 {
                return;
            }
        }
    }
}

fn is_code_file(ext: &str) -> bool {
    matches!(
        ext,
        "rs" | "ts" | "tsx" | "js" | "jsx" | "go" | "py" | "java" | "c" | "h" | "cpp" | "hpp" | "rb" | "zig"
    )
}

fn looks_like_definition(line: &str, ext: &str) -> bool {
    let trimmed = line.trim_start();
    match ext {
        "rs" => {
            trimmed.starts_with("fn ")
                || trimmed.starts_with("pub fn ")
                || trimmed.starts_with("pub(crate) fn ")
                || trimmed.starts_with("struct ")
                || trimmed.starts_with("pub struct ")
                || trimmed.starts_with("enum ")
                || trimmed.starts_with("pub enum ")
                || trimmed.starts_with("trait ")
                || trimmed.starts_with("pub trait ")
                || trimmed.starts_with("impl ")
                || trimmed.starts_with("mod ")
                || trimmed.starts_with("pub mod ")
                || trimmed.starts_with("const ")
                || trimmed.starts_with("pub const ")
                || trimmed.starts_with("type ")
                || trimmed.starts_with("pub type ")
        }
        "go" => trimmed.starts_with("func ") || trimmed.starts_with("type "),
        "py" => trimmed.starts_with("def ") || trimmed.starts_with("class "),
        "ts" | "tsx" | "js" | "jsx" => {
            trimmed.starts_with("function ")
                || trimmed.starts_with("export function ")
                || trimmed.starts_with("class ")
                || trimmed.starts_with("export class ")
                || trimmed.starts_with("interface ")
                || trimmed.starts_with("export interface ")
                || trimmed.starts_with("const ")
                || trimmed.starts_with("export const ")
        }
        "java" => {
            (trimmed.contains("class ") || trimmed.contains("interface "))
                || (trimmed.contains('(') && !trimmed.starts_with("//") && !trimmed.starts_with('*'))
        }
        _ => trimmed.starts_with("fn ") || trimmed.starts_with("def ") || trimmed.starts_with("func "),
    }
}
