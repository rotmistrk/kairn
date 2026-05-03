//! TypeScript/JavaScript import navigation — scans for exports.

use std::path::Path;

use super::{ImportSymbol, LanguageNav, SymbolKind};

/// TypeScript/JavaScript language navigator.
pub struct TsNav;

impl LanguageNav for TsNav {
    fn extensions(&self) -> &[&str] {
        &["ts", "tsx", "js", "jsx"]
    }

    fn scan_exports(&self, path: &Path, content: &str) -> Vec<ImportSymbol> {
        let module = module_name(path);
        let mut symbols = Vec::new();
        for (idx, line) in content.lines().enumerate() {
            let trimmed = line.trim();
            if let Some(sym) = parse_ts_export(trimmed) {
                symbols.push(ImportSymbol {
                    name: sym.0.to_string(),
                    module_path: module.clone(),
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
                if t.starts_with("import ") {
                    extract_ts_import_path(t)
                } else {
                    None
                }
            })
            .collect()
    }
}

fn parse_ts_export(line: &str) -> Option<(&str, SymbolKind)> {
    if !line.starts_with("export ") {
        return None;
    }
    let rest = line.strip_prefix("export ")?;
    let rest = rest.strip_prefix("default ").unwrap_or(rest);
    if let Some(r) = rest.strip_prefix("function ") {
        Some((extract_ts_name(r), SymbolKind::Function))
    } else if let Some(r) = rest.strip_prefix("class ") {
        Some((extract_ts_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("interface ") {
        Some((extract_ts_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("type ") {
        Some((extract_ts_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("enum ") {
        Some((extract_ts_name(r), SymbolKind::Type))
    } else if let Some(r) = rest.strip_prefix("const ") {
        Some((extract_ts_name(r), SymbolKind::Constant))
    } else if let Some(r) = rest.strip_prefix("let ") {
        Some((extract_ts_name(r), SymbolKind::Constant))
    } else {
        None
    }
}

fn extract_ts_name(s: &str) -> &str {
    let end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
        .unwrap_or(s.len());
    &s[..end]
}

fn extract_ts_import_path(line: &str) -> Option<String> {
    // import ... from "path" or import "path"
    let from_idx = line.find("from ")?;
    let rest = &line[from_idx + 5..];
    let quote = rest.find(['\'', '"'])?;
    let start = quote + 1;
    let end = rest[start..].find(['\'', '"'])?;
    Some(rest[start..start + end].to_string())
}

fn module_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string()
}
