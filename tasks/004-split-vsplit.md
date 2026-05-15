# :split / :vsplit

<!-- TODO: mark done → todo tree [2][3] -->

## Overview

Split the center panel to show multiple editor buffers simultaneously.
`:split` splits horizontally (top/bottom), `:vsplit` splits vertically
(left/right).

## Design

The center panel currently holds a single TabGroup. Splitting means the
center panel becomes a nested LayoutGroup:

```
Center (SplitGroup)
├── EditorView (top or left)
└── EditorView (bottom or right)
```

### Commands

- `:split [file]` — horizontal split (new pane below)
- `:vsplit [file]` — vertical split (new pane to the right)
- `:only` — close all splits, keep focused one
- `Ctrl-W h/j/k/l` — navigate between splits (vim convention)
- `Ctrl-W =` — equalize split sizes
- `Ctrl-W +/-` — grow/shrink focused split

### Behavior

- Each split pane is an independent editor with its own cursor/mode
- Same file can be open in multiple splits (shared buffer, independent views)
- Tab switching (M-0..9) applies to the focused split pane
- `:q` in a split closes that pane (not the whole center)

## Architecture

Option A: Center panel becomes a recursive SplitGroup that holds either
a TabGroup or two SplitGroups (binary tree of splits).

Option B: Center panel holds a flat list of editor views with a layout
strategy (simpler, max 2-4 splits).

Recommend Option A for flexibility, but start with max 2 splits for v1.

## Implementation Order

1. `:split` — horizontal, same file in both panes
2. `:split file` — horizontal, new file in bottom pane
3. `Ctrl-W j/k` — navigate between horizontal splits
4. `:only` — collapse splits
5. `:vsplit` — vertical split
6. `Ctrl-W h/l` — navigate vertical splits
7. Resize with `Ctrl-W +/-/=`

## Constraints

- Must integrate with session persistence (save/restore split state)
- Must integrate with MCP (list_tabs shows split panes)
- 240 code line max per file
- Zoom (F5) should maximize the focused split pane
