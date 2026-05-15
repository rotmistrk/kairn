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

    // Some tools (e.g. jdtls) don't support --version and start a long-running
    // server instead. Use a timeout to avoid hanging.
    let version = Command::new(name)
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()
        .and_then(|mut child| {
            let timeout = std::time::Duration::from_secs(2);
            let start = std::time::Instant::now();
            loop {
                match child.try_wait() {
                    Ok(Some(status)) if status.success() => {
                        let out = child.wait_with_output().ok()?;
                        return String::from_utf8(out.stdout)
                            .ok()
                            .and_then(|s| s.lines().next().map(|l| l.trim().to_string()));
                    }
                    Ok(Some(_)) => return None,
                    Ok(None) => {
                        if start.elapsed() > timeout {
                            child.kill().ok();
                            child.wait().ok();
                            return None;
                        }
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                    Err(_) => return None,
                }
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

    #[test]
    fn check_all_tools_includes_all_known_tools() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("main.rs"), "fn main() {}").unwrap();
        let results = check_all_tools(dir.path());
        let names: Vec<&str> = results.iter().map(|t| t.name).collect();
        // Should include all TOOLS entries, not just relevant ones
        for &(tool_name, _) in TOOLS {
            assert!(names.contains(&tool_name), "missing tool: {tool_name}");
        }
        // Relevant tools should come first (kiro-cli, rust-analyzer for .rs)
        assert_eq!(results[0].name, "kiro-cli");
    }
}
