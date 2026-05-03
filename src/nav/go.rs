//! Go import navigation — scans for exported (capitalized) symbols.

use std::path::Path;

use super::{ImportSymbol, LanguageNav, SymbolKind};

/// Go language navigator.
pub struct GoNav;

impl LanguageNav for GoNav {
    fn extensions(&self) -> &[&str] {
        &["go"]
    }

    fn scan_exports(&self, path: &Path, content: &str) -> Vec<ImportSymbol> {
        let pkg = extract_package(content);
        let mut symbols = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(sym) = parse_go_export(trimmed) {
                if is_exported(sym.0) {
                    symbols.push(ImportSymbol {
                        name: sym.0.to_string(),
                        module_path: pkg.clone(),
                        file: path.to_path_buf(),
                        line: idx + 1,
                        kind: sym.1,
                    });
                }
            }
        }
        symbols
    }

    fn scan_imports(&self, content: &str) -> Vec<String> {
        let mut imports = Vec::new();
        let mut in_block = false;
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with("import (") {
                in_block = true;
                continue;
            }
            if in_block {
                if t == ")" {
                    in_block = false;
                    continue;
                }
                let cleaned = t.trim_matches('"').to_string();
                if !cleaned.is_empty() {
                    imports.push(cleaned);
                }
            } else if let Some(rest) = t.strip_prefix("import ") {
                let cleaned = rest.trim_matches('"').to_string();
                if !cleaned.is_empty() {
                    imports.push(cleaned);
                }
            }
        }
        imports
    }
}

fn extract_package(content: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("package ") {
            return rest.trim().to_string();
        }
    }
    String::new()
}

fn parse_go_export(line: &str) -> Option<(&str, SymbolKind)> {
    if let Some(r) = line.strip_prefix("func ") {
        Some((extract_go_name(r), SymbolKind::Function))
    } else if let Some(r) = line.strip_prefix("type ") {
        Some((extract_go_name(r), SymbolKind::Type))
    } else if line.starts_with("const ") || line.starts_with("var ") {
        let r = line
            .strip_prefix("const ")
            .or_else(|| line.strip_prefix("var "))?;
        Some((extract_go_name(r), SymbolKind::Constant))
    } else {
        None
    }
}

fn extract_go_name(s: &str) -> &str {
    // Skip receiver: (r *Receiver) Name(...)
    let s = if s.starts_with('(') {
        s.find(')').map(|i| s[i + 1..].trim()).unwrap_or(s)
    } else {
        s
    };
    let end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(s.len());
    &s[..end]
}

fn is_exported(name: &str) -> bool {
    name.starts_with(|c: char| c.is_uppercase())
}
