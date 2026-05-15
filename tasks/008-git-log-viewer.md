# Git Commit Log Viewer

<!-- TODO: mark done → todo tree [2][7] -->

## Overview

A scrollable commit log view showing project history. Opens as a tab in
the center or tools panel.

## Design

### Display

```
 * a1b2c3d (HEAD -> main) Fix reflow bug        John  2h ago
 * f4e5d6c Add blame mode                       Alice 1d ago
 * 9876543 Refactor layout                      John  3d ago
```

- One line per commit: graph marker, short hash, decorations, subject, author, relative date
- Scrollable with j/k
- Enter → show full commit details (message + diffstat)
- `d` → show full diff for that commit

### Commands

- `:log` — open log for the whole repo
- `:log %` — open log for current file (file history)

## Implementation

1. Shell out to `git log --oneline --decorate --graph -n 200` (or structured format)
2. Parse into `Vec<CommitEntry>`
3. Render in a ResultsView-style scrollable list
4. On Enter: `git show <hash>` → display in a read-only editor tab

## Constraints

- Async: don't block UI while loading
- Limit initial load to 200 commits, load more on scroll-to-bottom
- 240 code line max per file
