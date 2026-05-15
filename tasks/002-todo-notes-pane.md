# Todo Notes Pane

<!-- TODO: mark done → todo tree [2][1] -->

## Overview

Open a dedicated editor tab in the center panel that displays and edits the
`note` field of the currently selected todo item. Syncs on cursor movement
in the todo tree.

## Design

- "Notes" tab in center panel (reuses EditorView with in-memory buffer)
- On todo tree cursor move → update Notes content from item.note
- On editor save (`:w`) → write buffer back to TodoItem.note, persist .kairn.todo
- If no item selected or item has no note → show empty buffer (editable)

## Implementation

1. New command: `CM_TODO_CURSOR_MOVED` emitted by todo tree on j/k navigation
2. Handler opens/updates a "Notes" editor tab with the item's note content
3. EditorView already supports `from_text()` for in-memory buffers
4. On save → find the corresponding TodoItem by path, update .note field
5. Save the todo file

## Keybinding

- `Enter` on a todo item (or a dedicated key like `N`) opens/focuses the Notes tab

## Constraints

- Reuse existing EditorView (vim keybindings, syntax: markdown)
- 240 code line max per file
- Notes tab is closeable (re-opens on next cursor move if needed)
