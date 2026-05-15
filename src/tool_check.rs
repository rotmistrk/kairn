//! Tool checker — detects installed dev tools relevant to the project.

use std::path::Path;
use std::process::Command;

pub struct ToolStatus {
    pub name: &'static str,
    pub found: bool,
    pub version: Option<String>,
    pub install_hint: &'static str,
}

/// Tools to check with their binary name and install hint.
const TOOLS: &[(&str, &str)] = &[
    ("kiro-cli", "cargo install kiro-cli"),
    ("rust-analyzer", "rustup component add rust-analyzer"),
    ("gopls", "go install golang.org/x/tools/gopls@latest"),
    (
        "typescript-language-server",
        "npm i -g typescript-language-server typescript",
    ),
    ("clangd", "brew install llvm (macOS) / apt install clangd"),
    ("pyright-langserver", "npm i -g pyright"),
    ("jdtls", "brew install jdtls / manual install"),
];

/// Map from file extension to required tool binary name.
const EXT_TO_TOOL: &[(&str, &str)] = &[
    ("rs", "rust-analyzer"),
    ("go", "gopls"),
    ("ts", "typescript-language-server"),
    ("tsx", "typescript-language-server"),
    ("js", "typescript-language-server"),
    ("jsx", "typescript-language-server"),
    ("c", "clangd"),
    ("cpp", "clangd"),
    ("h", "clangd"),
    ("py", "pyright-langserver"),
    ("java", "jdtls"),
];

/// Detect which tools are relevant and their status.
/// Get install hint for a tool by command name.
pub fn install_hint(command: &str) -> &'static str {
    TOOLS
        .iter()
        .find(|(name, _)| *name == command)
        .map(|(_, hint)| *hint)
        .unwrap_or("check your package manager")
}

pub fn check_tools(root_dir: &Path) -> Vec<ToolStatus> {
    let relevant = relevant_tools(root_dir);
    relevant
        .into_iter()
        .map(|(name, hint)| {
            let (found, version) = probe(name);
            ToolStatus {
                name,
                found,
                version,
                install_hint: hint,
            }
        })
        .collect()
}

/// Check ALL known tools (for welcome screen): relevant first, then others.
pub fn check_all_tools(root_dir: &Path) -> Vec<ToolStatus> {
    let relevant = relevant_tools(root_dir);
    let relevant_names: Vec<&str> = relevant.iter().map(|(n, _)| *n).collect();
    let mut results: Vec<ToolStatus> = relevant
        .iter()
        .map(|(name, hint)| {
            let (found, version) = probe(name);
            ToolStatus {
                name,
                found,
                version,
                install_hint: hint,
            }
        })
        .collect();
    for &(name, hint) in TOOLS {
        if !relevant_names.contains(&name) {
            let (found, version) = probe(name);
            results.push(ToolStatus {
                name,
                found,
                version,
                install_hint: hint,
            });
        }
    }
    results
}

/// Determine which tools are relevant based on file extensions in the project.
fn relevant_tools(root_dir: &Path) -> Vec<(&'static str, &'static str)> {
    let mut needed: Vec<&str> = vec!["kiro-cli"];

    // Scan top-level and src/ for relevant extensions
    let dirs_to_scan: Vec<std::path::PathBuf> = vec![root_dir.to_path_buf(), root_dir.join("src")];

    for dir in dirs_to_scan {
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    for &(e, tool) in EXT_TO_TOOL {
                        if ext == e && !needed.contains(&tool) {
                            needed.push(tool);
                        }
                    }
                }
            }
        }
    }

    TOOLS
        .iter()
        .filter(|(name, _)| needed.contains(name))
        .copied()
        .collect()
}

/// Check if a tool exists and get its version.
fn probe(name: &str) -> (bool, Option<String>) {
    let found = Command::new("which")
        .arg(name)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if !found {
        return (false, None);
    }

    let version = Command::new(name).arg("--version").output().ok().and_then(|o| {
        if o.status.success() {
            String::from_utf8(o.stdout)
                .ok()
                .and_then(|s| s.lines().next().map(|l| l.trim().to_string()))
        } else {
            None
        }
    });

    (true, version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn relevant_tools_always_includes_kiro_cli() {
        let tools = relevant_tools(&PathBuf::from("/nonexistent"));
        let names: Vec<&str> = tools.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"kiro-cli"));
    }

    #[test]
    fn probe_nonexistent_tool() {
        let (found, version) = probe("definitely-not-a-real-tool-xyz");
        assert!(!found);
        assert!(version.is_none());
    }
}
