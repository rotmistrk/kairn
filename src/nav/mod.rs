//! Language-aware import navigation.
//!
//! Scans workspace for importable symbols and provides go-to-definition
//! for imports without LSP. Each language implements [`LanguageNav`].

mod go;
mod java;
mod rust_nav;
mod ts;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A symbol that can be imported.
#[derive(Debug, Clone)]
pub struct ImportSymbol {
    /// Symbol name (e.g. `HashMap`, `fmt.Println`).
    pub name: String,
    /// Module/package path (e.g. `std::collections`, `fmt`).
    pub module_path: String,
    /// File where the symbol is defined.
    pub file: PathBuf,
    /// 1-based line number of the definition.
    pub line: usize,
    /// The kind of symbol.
    pub kind: SymbolKind,
}

/// Classification of an importable symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    /// A type (struct, class, interface, enum).
    Type,
    /// A function or method.
    Function,
    /// A constant or static value.
    Constant,
    /// A module or package.
    Module,
}

/// Result of resolving an import.
#[derive(Debug, Clone)]
pub struct ImportTarget {
    /// File containing the definition.
    pub file: PathBuf,
    /// 1-based line number.
    pub line: usize,
    /// The resolved symbol.
    pub symbol: String,
}

/// Language-specific import navigation.
pub trait LanguageNav: Send + Sync {
    /// File extensions this language handles.
    fn extensions(&self) -> &[&str];

    /// Scan a file and return importable symbols it exports.
    fn scan_exports(&self, path: &Path, content: &str) -> Vec<ImportSymbol>;

    /// Extract import statements from a file.
    fn scan_imports(&self, content: &str) -> Vec<String>;
}

/// Index of importable symbols across the workspace.
pub struct ImportIndex {
    /// Symbol name → list of definitions.
    symbols: HashMap<String, Vec<ImportSymbol>>,
    /// Language navigators.
    languages: Vec<Box<dyn LanguageNav>>,
}

impl ImportIndex {
    /// Create a new index with all supported languages.
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            languages: vec![
                Box::new(rust_nav::RustNav),
                Box::new(go::GoNav),
                Box::new(java::JavaNav),
                Box::new(ts::TsNav),
            ],
        }
    }

    /// Scan all files under `root` and build the symbol index.
    pub fn build(&mut self, root: &Path) {
        self.symbols.clear();
        let walker = ignore::WalkBuilder::new(root).hidden(true).build();
        for entry in walker.flatten() {
            if !entry.file_type().is_some_and(|ft| ft.is_file()) {
                continue;
            }
            self.index_file(entry.path());
        }
    }

    /// Look up symbols by name.
    pub fn lookup(&self, name: &str) -> &[ImportSymbol] {
        self.symbols.get(name).map(|v| v.as_slice()).unwrap_or(&[])
    }

    /// Resolve an import string to a file location.
    pub fn resolve(&self, import: &str) -> Option<&ImportSymbol> {
        // Try exact match first, then last segment
        if let Some(syms) = self.symbols.get(import) {
            return syms.first();
        }
        let last = import.rsplit(&['.', ':', '/'][..]).next()?;
        self.symbols
            .get(last)
            .and_then(|syms| syms.iter().find(|s| s.module_path.contains(import)))
    }

    fn index_file(&mut self, path: &Path) {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let nav = match self.find_nav(ext) {
            Some(n) => n,
            None => return,
        };
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => return,
        };
        let exports = nav.scan_exports(path, &content);
        for sym in exports {
            self.symbols.entry(sym.name.clone()).or_default().push(sym);
        }
    }

    fn find_nav(&self, ext: &str) -> Option<&dyn LanguageNav> {
        self.languages
            .iter()
            .find(|n| n.extensions().contains(&ext))
            .map(|n| n.as_ref())
    }
}

impl Default for ImportIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn index_rust_file() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("lib.rs"),
            "pub struct Foo;\npub fn bar() {}\n",
        )
        .unwrap();
        let mut idx = ImportIndex::new();
        idx.build(dir.path());
        assert!(!idx.lookup("Foo").is_empty());
        assert!(!idx.lookup("bar").is_empty());
    }

    #[test]
    fn index_go_file() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("main.go"),
            "package main\n\nfunc Hello() {}\n\ntype Config struct {}\n",
        )
        .unwrap();
        let mut idx = ImportIndex::new();
        idx.build(dir.path());
        assert!(!idx.lookup("Hello").is_empty());
        assert!(!idx.lookup("Config").is_empty());
    }

    #[test]
    fn lookup_missing_returns_empty() {
        let idx = ImportIndex::new();
        assert!(idx.lookup("Nonexistent").is_empty());
    }
}
