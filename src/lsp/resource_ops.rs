//! Resource operations for LSP workspace edits (rename, create, delete files).

use serde_json::Value;

/// Handle a resource operation (rename, create, delete). Returns true on success.
pub fn apply_resource_op(kind: &str, op: &Value) -> bool {
    match kind {
        "rename" => {
            let old = op.get("oldUri").and_then(|u| u.as_str()).unwrap_or("");
            let new = op.get("newUri").and_then(|u| u.as_str()).unwrap_or("");
            if old.is_empty() || new.is_empty() {
                return false;
            }
            let old_path = uri_to_path(old);
            let new_path = uri_to_path(new);
            rename_file(&old_path, &new_path)
        }
        "create" => {
            let uri = op.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            if uri.is_empty() {
                return false;
            }
            create_file(&uri_to_path(uri))
        }
        "delete" => {
            let uri = op.get("uri").and_then(|u| u.as_str()).unwrap_or("");
            if uri.is_empty() {
                return false;
            }
            delete_file(&uri_to_path(uri))
        }
        _ => false,
    }
}

fn rename_file(old: &str, new: &str) -> bool {
    if let Some(parent) = std::path::Path::new(new).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    // Try git mv first if file is in a git repo
    if is_in_git_repo(old) && git_mv(old, new) {
        return true;
    }
    std::fs::rename(old, new).is_ok()
}

/// Check if a path is inside a git repository.
fn is_in_git_repo(path: &str) -> bool {
    let mut dir = std::path::Path::new(path).to_path_buf();
    loop {
        if dir.join(".git").exists() {
            return true;
        }
        if !dir.pop() {
            return false;
        }
    }
}

/// Attempt to rename using `git mv`. Returns true on success.
fn git_mv(old: &str, new: &str) -> bool {
    std::process::Command::new("git")
        .args(["mv", old, new])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn create_file(path: &str) -> bool {
    if let Some(parent) = std::path::Path::new(path).parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    std::fs::File::create(path).is_ok()
}

fn delete_file(path: &str) -> bool {
    let p = std::path::Path::new(path);
    if p.is_dir() {
        std::fs::remove_dir_all(p).is_ok()
    } else {
        std::fs::remove_file(p).is_ok()
    }
}

fn uri_to_path(uri: &str) -> String {
    uri.strip_prefix("file://").unwrap_or(uri).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn rename_moves_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let old = dir.path().join("Old.java");
        std::fs::write(&old, "class Old {}").expect("write");
        let op = json!({
            "kind": "rename",
            "oldUri": format!("file://{}", old.display()),
            "newUri": format!("file://{}", dir.path().join("New.java").display())
        });
        assert!(apply_resource_op("rename", &op));
        assert!(!old.exists());
        assert!(dir.path().join("New.java").exists());
    }

    #[test]
    fn create_makes_file_and_dirs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let target = dir.path().join("a").join("b").join("New.java");
        let op = json!({"kind": "create", "uri": format!("file://{}", target.display())});
        assert!(apply_resource_op("create", &op));
        assert!(target.exists());
    }

    #[test]
    fn delete_removes_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("gone.txt");
        std::fs::write(&file, "x").expect("write");
        let op = json!({"kind": "delete", "uri": format!("file://{}", file.display())});
        assert!(apply_resource_op("delete", &op));
        assert!(!file.exists());
    }

    #[test]
    fn unknown_kind_returns_false() {
        assert!(!apply_resource_op("unknown", &json!({})));
    }

    #[test]
    fn rename_uses_git_mv_in_repo() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Initialize a git repo
        let status = std::process::Command::new("git")
            .args(["init"])
            .current_dir(dir.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        if status.map(|s| s.success()).unwrap_or(false) {
            let old = dir.path().join("Tracked.java");
            std::fs::write(&old, "class Tracked {}").expect("write");
            // Stage the file
            let _ = std::process::Command::new("git")
                .args(["add", "Tracked.java"])
                .current_dir(dir.path())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
            let new = dir.path().join("Renamed.java");
            let op = json!({
                "kind": "rename",
                "oldUri": format!("file://{}", old.display()),
                "newUri": format!("file://{}", new.display())
            });
            assert!(apply_resource_op("rename", &op));
            assert!(!old.exists());
            assert!(new.exists());
            // Verify git knows about the rename
            let output = std::process::Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(dir.path())
                .output()
                .expect("git status");
            let status_text = String::from_utf8_lossy(&output.stdout);
            assert!(
                status_text.contains("R") || status_text.contains("renamed"),
                "git should track rename: {}",
                status_text
            );
        }
    }
}
