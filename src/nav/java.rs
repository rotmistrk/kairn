//! Java import navigation — scans for public classes and methods.

use std::path::Path;

use super::{ImportSymbol, LanguageNav, SymbolKind};

/// Java language navigator.
pub struct JavaNav;

impl LanguageNav for JavaNav {
    fn extensions(&self) -> &[&str] {
        &["java"]
    }

    fn scan_exports(&self, path: &Path, content: &str) -> Vec<ImportSymbol> {
        let pkg = extract_java_package(content);
        let mut symbols = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(sym) = parse_java_public(trimmed) {
                symbols.push(ImportSymbol {
                    name: sym.0.to_string(),
                    module_path: pkg.clone(),
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
                t.strip_prefix("import ").map(|rest| {
                    rest.trim_start_matches("static ")
                        .trim_end_matches(';')
                        .to_string()
                })
            })
            .collect()
    }
}

fn extract_java_package(content: &str) -> String {
    for line in content.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("package ") {
            return rest.trim_end_matches(';').trim().to_string();
        }
    }
    String::new()
}

fn parse_java_public(line: &str) -> Option<(&str, SymbolKind)> {
    if !line.starts_with("public ") {
        return None;
    }
    let rest = line.strip_prefix("public ")?;
    let rest = rest
        .strip_prefix("static ")
        .or_else(|| rest.strip_prefix("abstract "))
        .or_else(|| rest.strip_prefix("final "))
        .unwrap_or(rest);
    if let Some(r) = rest.strip_prefix("class ") {
        Some((extract_java_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("interface ") {
        Some((extract_java_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("enum ") {
        Some((extract_java_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("record ") {
        Some((extract_java_name(r), SymbolKind::Type))
    } else {
        None
    }
}

fn extract_java_name(s: &str) -> &str {
    let end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(s.len());
    &s[..end]
}
