//! Project root detection — walk up from a file looking for project markers.
//! Supports user override via a `project-root` Tcl proc in config.

use std::path::{Path, PathBuf};

use rusticle::interpreter::Interpreter;

/// Markers that indicate a project root (checked in order).
const MARKERS: &[&str] = &[
    ".git",
    ".kairn",
    "Cargo.toml",
    "Makefile",
    "package.json",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "CMakeLists.txt",
    ".hg",
    "pyproject.toml",
];

/// Detect project root, checking user Tcl proc first, then built-in heuristic.
/// The user can define `proc project-root {path} { ... }` in their config.
pub fn detect_project_root(start: &Path, fallback: &Path) -> PathBuf {
    // Try user-defined Tcl proc from global config
    if let Some(root) = try_tcl_project_root(start) {
        return root;
    }
    detect_project_root_builtin(start, fallback)
}

/// Built-in heuristic: walk up looking for marker files/dirs.
pub fn detect_project_root_builtin(start: &Path, fallback: &Path) -> PathBuf {
    let mut dir = if start.is_file() {
        start.parent().unwrap_or(start)
    } else {
        start
    };
    loop {
        for marker in MARKERS {
            if dir.join(marker).exists() {
                return dir.to_path_buf();
            }
        }
        match dir.parent() {
            Some(parent) if parent != dir => dir = parent,
            _ => break,
        }
    }
    fallback.to_path_buf()
}

/// Try calling user's `project-root` proc from global config.
fn try_tcl_project_root(start: &Path) -> Option<PathBuf> {
    let config_path = global_config_path()?;
    if !config_path.exists() {
        return None;
    }
    let script = std::fs::read_to_string(&config_path).ok()?;
    // Quick check: does the config even define project-root?
    if !script.contains("project-root") {
        return None;
    }
    let mut interp = Interpreter::new();
    interp.eval(&script).ok()?;
    // Call the proc with the file path
    let call = format!("project-root {{{}}}", start.display());
    let result = interp.eval(&call).ok()?;
    let path = PathBuf::from(result.as_str().trim());
    if path.is_dir() {
        Some(path)
    } else {
        None
    }
}

fn global_config_path() -> Option<PathBuf> {
    let config_dir = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg)
    } else {
        PathBuf::from(std::env::var("HOME").ok()?).join(".config")
    };
    Some(config_dir.join("kairn").join("init.tcl"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn finds_git_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src/deep")).unwrap();
        fs::create_dir(root.join(".git")).unwrap();
        fs::write(root.join("src/deep/main.rs"), "").unwrap();

        let file = root.join("src/deep/main.rs");
        assert_eq!(detect_project_root_builtin(&file, file.parent().unwrap()), root);
    }

    #[test]
    fn finds_cargo_toml_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "").unwrap();
        fs::write(root.join("src/lib.rs"), "").unwrap();

        let file = root.join("src/lib.rs");
        assert_eq!(detect_project_root_builtin(&file, file.parent().unwrap()), root);
    }

    #[test]
    fn falls_back_to_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/file.txt"), "").unwrap();

        let file = root.join("sub/file.txt");
        let parent = file.parent().unwrap();
        assert_eq!(detect_project_root_builtin(&file, parent), parent);
    }

    #[test]
    fn prefers_nearest_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("workspace/sub/src")).unwrap();
        fs::create_dir(root.join("workspace/.git")).unwrap();
        fs::write(root.join("workspace/sub/Cargo.toml"), "").unwrap();
        fs::write(root.join("workspace/sub/src/main.rs"), "").unwrap();

        let file = root.join("workspace/sub/src/main.rs");
        // Should find Cargo.toml in sub/ (nearest), not .git in workspace/
        assert_eq!(
            detect_project_root(&file, file.parent().unwrap()),
            root.join("workspace/sub")
        );
    }
}
