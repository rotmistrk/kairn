//! Rust import navigation — scans for `pub` items.

use std::path::Path;

use super::{ImportSymbol, LanguageNav, SymbolKind};

/// Rust language navigator.
pub struct RustNav;

impl LanguageNav for RustNav {
    fn extensions(&self) -> &[&str] {
        &["rs"]
    }

    fn scan_exports(&self, path: &Path, content: &str) -> Vec<ImportSymbol> {
        let mut symbols = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(sym) = parse_pub_item(trimmed) {
                symbols.push(ImportSymbol {
                    name: sym.0.to_string(),
                    module_path: module_path(path),
                    file: path.to_path_buf(),
                    line: idx + 1,
                    kind: sym.1,
                });
            }
        }
        symbols
    }

    fn scan_imports(&self, content: &str) -> Vec<String> {
        content
            .lines()
            .filter_map(|line| {
                let t = line.trim();
                if t.starts_with("use ") {
                    Some(
                        t.trim_start_matches("use ")
                            .trim_end_matches(';')
                            .to_string(),
                    )
                } else {
                    None
                }
            })
            .collect()
    }
}

fn parse_pub_item(line: &str) -> Option<(&str, SymbolKind)> {
    if !line.starts_with("pub ") {
        return None;
    }
    let rest = line.strip_prefix("pub ")?;
    let rest = rest
        .strip_prefix("(crate) ")
        .or_else(|| rest.strip_prefix("(super) "))
        .unwrap_or(rest);
    if let Some(r) = rest.strip_prefix("struct ") {
        Some((extract_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("enum ") {
        Some((extract_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("trait ") {
        Some((extract_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("type ") {
        Some((extract_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("fn ") {
        Some((extract_name(r), SymbolKind::Function))
    } else if let Some(r) = rest.strip_prefix("const ") {
        Some((extract_name(r), SymbolKind::Constant))
    } else if let Some(r) = rest.strip_prefix("static ") {
        Some((extract_name(r), SymbolKind::Constant))
    } else if let Some(r) = rest.strip_prefix("mod ") {
        Some((extract_name(r), SymbolKind::Module))
    } else {
        None
    }
}

fn extract_name(s: &str) -> &str {
    let end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(s.len());
    &s[..end]
}

fn module_path(path: &Path) -> String {
    path.to_string_lossy().replace('/', "::").replace(".rs", "")
}
