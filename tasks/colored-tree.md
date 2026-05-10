# Task: Colored File Tree with Git Status

## Overview

Add git status colors to the file tree and fix cursor styling.

## Cursor Style

Change tree cursor from reverse (inverts colors) to:
- Background: dark blue (Ansi 4 bg)
- Underline: true
- Foreground: UNCHANGED (preserve git status color)

This is in `txv-widgets/src/tree_view.rs` where the cursor row is drawn.

## Directory Color

- All directories: light blue (Ansi 14 fg)
- Already distinguished by expand/collapse icon (▸/▾)

## Git Status Colors

| Status | Fg Color | Ansi |
|--------|----------|------|
| Directory | Light blue | 14 |
| Clean file | Default | 7 |
| Modified (unstaged changes) | Blue | 12 |
| Added (staged, new to git) | Green | 2 |
| Untracked (not in git) | Red/brown | 1 |
| Ignored (.gitignore) | Dark gray | 8 |

## Git Detection

1. Shell out to `git status --porcelain=v1 -z` from the workspace root
2. Parse output: `XY path\0` format where X=index status, Y=worktree status
3. Map statuses:
   - `??` → Untracked
   - `A ` or `A?` → Added
   - ` M` or `MM` → Modified
   - `!!` → Ignored (need `--ignored` flag)
4. Cache results in `FileTreeView`
5. Refresh on tree refresh tick (every 60 ticks)
6. Directories inherit the "most important" child status (untracked > modified > added > clean)

## Editor Tab Dirty Marker

- When buffer is dirty (unsaved), prepend `*` to tab title: `*main.rs`
- When saved (or autosave fires), remove the `*`
- This is in `src/views/editor/mod.rs` — override `title()` method

## Implementation Files

- `txv-widgets/src/tree_view.rs` — cursor style change
- `txv-widgets/src/file_tree.rs` — add git_status field, color per node
- `src/views/tree.rs` — git status collection, refresh integration
- `src/views/editor/mod.rs` — dirty marker in title()

## Git Status Collection (new file: `src/git_status.rs`)

```rust
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

pub enum FileStatus { Clean, Modified, Added, Untracked, Ignored }

pub fn collect_git_status(root: &Path) -> HashMap<String, FileStatus> {
    // Run: git -C {root} status --porcelain=v1 -z --ignored
    // Parse output, return relative path → status map
}
```

## Constraints

- Pre-commit hook MUST pass
- 240 code line max per file
- No unwrap/expect/panic in runtime code
- If `git` is not available or not a git repo, gracefully degrade (no colors, no crash)
- Tests: unit test for git status parsing, scenario test for tree colors

## Testing

- Unit test: parse_git_status("?? new.rs\0 M changed.rs\0") → correct HashMap
- Scenario test: create temp git repo, add files, verify tree shows colors
  (check cell styles at tree positions using surface inspection)

## References

- `.kiro/steering/steering.md` — SOPs
- Legacy implementation: `git show origin/legacy:src/panel/file_tree.rs` (collect_git_status function)
- `git show origin/legacy:src/tree.rs` — tree node structure with git status
