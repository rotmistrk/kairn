# Git Blame

<!-- TODO: mark done → todo tree [2][4] -->

## Overview

Show per-line blame annotations alongside the editor content. Toggle with
`:blame` or a keybinding.

## Design

### Display

```
 a1b2c3d John D. 2026-05-01 │ 42 │ fn main() {
 a1b2c3d John D. 2026-05-01 │ 43 │     let x = 1;
 f4e5d6c Alice  2026-05-10 │ 44 │     println!("{x}");
```

- Left gutter: short hash, author (truncated), date
- Separator `│` between blame and line numbers
- Same commit ranges get dimmed (only first line of a range shows full info)

### Interaction

- `Enter` on a blame line → show full commit message in status/overlay
- `o` → open the file at that commit (read-only, diff-able)
- `q` or `:blame` again → exit blame mode

## Implementation

1. Use `git2` crate's `Repository::blame_file()` API (no shelling out — per steering doc)
2. Iterate `BlameHunk`s to build `Vec<BlameLine>` (hash, author, date, line range)
3. Render as an extra gutter column in the editor (like line numbers)
4. Blame data cached per file, invalidated on save
5. Run blame on a background thread (git2 is blocking), send result via channel

## Commands

- `:blame` — toggle blame mode for current file
- Tcl: `git blame` / `git blame off`

## Constraints

- Non-blocking: run git blame async, show "loading..." until ready
- Handle files not in git gracefully (show message, no crash)
- 240 code line max per file
