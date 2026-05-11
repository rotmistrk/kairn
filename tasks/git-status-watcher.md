# Task: Proper Git Status with File Watching

## Requirements (NON-NEGOTIABLE)

1. **No subprocess** — use git2 crate only (already done)
2. **Nested git roots** — detect `.git` dirs in subtrees, track status per repo
3. **File watcher** — use `notify` crate (kqueue/inotify) to detect changes from ANY process
4. **Conflict status** — FileStatus::Conflict with distinct color (magenta, Ansi 5)
5. **Reactive** — refresh git status when:
   - File watcher detects `.git/index` change (commit, stage, external git operation)
   - File watcher detects working tree file change (any process modifies a file)
   - On save (CM_SAVE) as immediate trigger
6. **Never poll on timer for git status** — only react to events

## Architecture

```
FileWatcher (background thread)
    ├── watches .git/index for each discovered git root
    ├── watches working tree for file modifications
    └── sends GitChanged event to main thread via channel

Main thread (on Tick):
    └── polls FileWatcher channel → if changed, refresh git status via git2
```

## Implementation

### Dependencies

```toml
notify = "6"   # Cross-platform file watcher (kqueue, inotify, etc.)
git2 = { version = "0.19", default-features = false }
```

### FileStatus enum (update)

```rust
pub enum FileStatus {
    Clean,
    Modified,
    Added,
    Untracked,
    Ignored,
    Conflict,  // NEW
}
```

### Color scheme (update)

| Status | Color |
|--------|-------|
| Conflict | Magenta (Ansi 5) |

### Git Root Discovery

- Walk the tree nodes looking for directories containing `.git`
- Each git root gets its own `git2::Repository` instance
- File paths are relative to their nearest git root

### File Watcher

```rust
pub struct GitWatcher {
    rx: mpsc::Receiver<()>,  // signal that something changed
    _watcher: notify::RecommendedWatcher,
}

impl GitWatcher {
    pub fn new(roots: &[PathBuf]) -> Option<Self>;
    pub fn has_changes(&self) -> bool;  // non-blocking poll
}
```

- Watch `.git/index` and `.git/refs` for each git root
- Watch the working tree (debounced — coalesce rapid changes)
- On any change: send signal through channel
- Main thread checks `has_changes()` on Tick — if true, re-run git2 status

### Integration in FileTreeView

```rust
struct FileTreeView {
    inner: TreeView<FileTreeData>,
    watcher: Option<GitWatcher>,
    // ...
}
```

On Tick:
- Check `watcher.has_changes()` — if true, refresh git status via git2
- This is CHEAP (no subprocess, just reads git index)

On startup:
- Discover git roots
- Start watcher
- Initial git status collection

## Constraints

- Pre-commit hook MUST pass
- 240 code line max per file
- No unwrap/expect/panic
- No subprocess (git binary) EVER
- File watcher must be non-blocking (background thread)
- Graceful: if notify fails (e.g., too many watches), degrade to no git colors

## Files

- `src/git_status.rs` — update: add Conflict, nested roots
- `src/git_watcher.rs` — NEW: file watcher wrapper
- `src/views/tree.rs` — integrate watcher, reactive refresh
- `Cargo.toml` — add `notify = "6"`
