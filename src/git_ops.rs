//! Git operations — stage, unstage, untrack, commit via git2.

use std::path::Path;

/// Stage a file (add to index).
pub fn git_stage(root: &Path, rel_path: &str) -> Result<(), String> {
    let repo = git2::Repository::discover(root).map_err(|e| format!("git open: {e}"))?;
    let mut index = repo.index().map_err(|e| format!("git index: {e}"))?;
    index
        .add_path(Path::new(rel_path))
        .map_err(|e| format!("git add: {e}"))?;
    index.write().map_err(|e| format!("git write index: {e}"))?;
    Ok(())
}

/// Unstage a file (reset index entry to HEAD).
pub fn git_unstage(root: &Path, rel_path: &str) -> Result<(), String> {
    let repo = git2::Repository::discover(root).map_err(|e| format!("git open: {e}"))?;
    let mut index = repo.index().map_err(|e| format!("git index: {e}"))?;
    let path = Path::new(rel_path);
    // Try to get the entry from HEAD tree
    let head_entry = repo
        .head()
        .ok()
        .and_then(|h| h.peel_to_tree().ok())
        .and_then(|t| t.get_path(path).ok());
    match head_entry {
        Some(entry) => {
            let ie = git2::IndexEntry {
                ctime: git2::IndexTime::new(0, 0),
                mtime: git2::IndexTime::new(0, 0),
                dev: 0,
                ino: 0,
                mode: entry.filemode() as u32,
                uid: 0,
                gid: 0,
                file_size: 0,
                id: entry.id(),
                flags: 0,
                flags_extended: 0,
                path: rel_path.as_bytes().to_vec(),
            };
            index.add(&ie).map_err(|e| format!("git reset entry: {e}"))?;
        }
        None => {
            // File not in HEAD — remove from index entirely
            index.remove_path(path).map_err(|e| format!("git remove: {e}"))?;
        }
    }
    index.write().map_err(|e| format!("git write index: {e}"))?;
    Ok(())
}

/// Untrack a file (remove from index without deleting working copy).
pub fn git_untrack(root: &Path, rel_path: &str) -> Result<(), String> {
    let repo = git2::Repository::discover(root).map_err(|e| format!("git open: {e}"))?;
    let mut index = repo.index().map_err(|e| format!("git index: {e}"))?;
    index
        .remove_path(Path::new(rel_path))
        .map_err(|e| format!("git remove: {e}"))?;
    index.write().map_err(|e| format!("git write index: {e}"))?;
    Ok(())
}

/// Commit the current index with the given message.
pub fn git_commit(root: &Path, message: &str) -> Result<(), String> {
    let repo = git2::Repository::discover(root).map_err(|e| format!("git open: {e}"))?;
    let sig = repo.signature().map_err(|e| format!("git signature: {e}"))?;
    let mut index = repo.index().map_err(|e| format!("git index: {e}"))?;
    let tree_oid = index.write_tree().map_err(|e| format!("git write tree: {e}"))?;
    let tree = repo.find_tree(tree_oid).map_err(|e| format!("git find tree: {e}"))?;
    let parents: Vec<git2::Commit> = match repo.head() {
        Ok(head) => {
            let commit = head.peel_to_commit().map_err(|e| format!("git head commit: {e}"))?;
            vec![commit]
        }
        Err(_) => vec![], // Initial commit
    };
    let parent_refs: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parent_refs)
        .map_err(|e| format!("git commit: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn init_repo() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        git2::Repository::init(dir.path()).unwrap();
        // Configure user for commits
        let repo = git2::Repository::open(dir.path()).unwrap();
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test").unwrap();
        config.set_str("user.email", "test@test.com").unwrap();
        dir
    }

    #[test]
    fn stage_new_file() {
        let dir = init_repo();
        fs::write(dir.path().join("a.txt"), "hello").unwrap();
        assert!(git_stage(dir.path(), "a.txt").is_ok());
        // Verify it's in the index
        let repo = git2::Repository::open(dir.path()).unwrap();
        let index = repo.index().unwrap();
        assert!(index.get_path(Path::new("a.txt"), 0).is_some());
    }

    #[test]
    fn unstage_removes_from_index() {
        let dir = init_repo();
        fs::write(dir.path().join("b.txt"), "data").unwrap();
        git_stage(dir.path(), "b.txt").unwrap();
        git_commit(dir.path(), "init").unwrap();
        // Modify and stage
        fs::write(dir.path().join("b.txt"), "changed").unwrap();
        git_stage(dir.path(), "b.txt").unwrap();
        // Unstage should reset to HEAD version
        assert!(git_unstage(dir.path(), "b.txt").is_ok());
    }

    #[test]
    fn unstage_new_file_removes_entirely() {
        let dir = init_repo();
        fs::write(dir.path().join("new.txt"), "x").unwrap();
        git_stage(dir.path(), "new.txt").unwrap();
        assert!(git_unstage(dir.path(), "new.txt").is_ok());
        let repo = git2::Repository::open(dir.path()).unwrap();
        let index = repo.index().unwrap();
        assert!(index.get_path(Path::new("new.txt"), 0).is_none());
    }

    #[test]
    fn untrack_removes_from_index() {
        let dir = init_repo();
        fs::write(dir.path().join("c.txt"), "data").unwrap();
        git_stage(dir.path(), "c.txt").unwrap();
        assert!(git_untrack(dir.path(), "c.txt").is_ok());
        let repo = git2::Repository::open(dir.path()).unwrap();
        let index = repo.index().unwrap();
        assert!(index.get_path(Path::new("c.txt"), 0).is_none());
        // File still exists on disk
        assert!(dir.path().join("c.txt").exists());
    }

    #[test]
    fn commit_creates_head() {
        let dir = init_repo();
        fs::write(dir.path().join("d.txt"), "data").unwrap();
        git_stage(dir.path(), "d.txt").unwrap();
        assert!(git_commit(dir.path(), "first commit").is_ok());
        let repo = git2::Repository::open(dir.path()).unwrap();
        let head = repo.head().unwrap();
        let commit = head.peel_to_commit().unwrap();
        assert_eq!(commit.message(), Some("first commit"));
    }

    #[test]
    fn stage_nonexistent_file_errors() {
        let dir = init_repo();
        let result = git_stage(dir.path(), "nope.txt");
        assert!(result.is_err());
    }
}
