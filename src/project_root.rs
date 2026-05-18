//! Project root detection — walk up from a file looking for project markers.

use std::path::{Path, PathBuf};

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

/// Walk up from `start` looking for a directory containing any marker.
/// Returns the first ancestor (or `start` itself) that contains a marker.
/// Falls back to `fallback` if no marker is found.
pub fn detect_project_root(start: &Path, fallback: &Path) -> PathBuf {
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
        assert_eq!(detect_project_root(&file, file.parent().unwrap()), root);
    }

    #[test]
    fn finds_cargo_toml_root() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("Cargo.toml"), "").unwrap();
        fs::write(root.join("src/lib.rs"), "").unwrap();

        let file = root.join("src/lib.rs");
        assert_eq!(detect_project_root(&file, file.parent().unwrap()), root);
    }

    #[test]
    fn falls_back_to_parent() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        fs::create_dir_all(root.join("sub")).unwrap();
        fs::write(root.join("sub/file.txt"), "").unwrap();

        let file = root.join("sub/file.txt");
        let parent = file.parent().unwrap();
        assert_eq!(detect_project_root(&file, parent), parent);
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
