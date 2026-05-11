# Task: LSP Gaps — Navigation, Rename, and Java Support

## Overview

Address gaps discovered during LSP integration that affect real-world usability,
particularly for Java (jdtls) but also cross-language.

## Steps

### 1. Jump to line/col after goto_definition (Small — 1h)

Currently `goto_definition` opens the file but lands at line 0.
Pass `(line, col)` through the command chain so the editor scrolls to the target.

- Extend `CM_OPEN_FILE_FOCUS` data to carry optional `(line, col)`
- After tab opens, send cursor-move to the editor

### 2. Handle `jdt://` URIs from jdtls (Medium — 3h)

jdtls returns `jdt://` URIs for decompiled JDK/dependency sources.
Options:
- Send `java/classFileContents` request to get source text
- Display in a read-only "[decompiled]" tab
- Graceful fallback: show "Source not available" if request fails

### 3. Handle RenameFile/CreateFile in workspace edits (Medium — 3h)

`documentChanges` can contain resource operations alongside text edits:
- `RenameFile { oldUri, newUri }` — move/rename file
- `CreateFile { uri }` — create new file
- `DeleteFile { uri }` — delete file

Implement in `workspace_edit.rs`. Process resource ops in order alongside text edits.

### 4. Prefer `git mv` for file renames (Small — 1h)

When handling `RenameFile` in a git repo:
- Detect git repo (check for `.git` or use `gix`)
- Use `git mv` if available, fall back to `std::fs::rename`
- Ensures rename history is preserved

### 5. Editor keybinding for lsp-rename with input prompt (Medium — 3h)

Add `lsp-rename` and `code-action` as editor-triggered commands:
- Keybinding (e.g., `F2` for rename, `Ctrl-.` for code action)
- For rename: show mini-prompt (like search `/`) to enter new name
- Pre-fill with word under cursor
- Dispatch `CM_LSP_RENAME` with the entered name

### 6. Post-rename hook for test class sync (Low priority — exploratory)

When a Java class is renamed, optionally rename corresponding test class.
This is IDE-specific behavior (not LSP). Consider:
- Convention-based: `Foo.java` → `FooTest.java`
- Prompt user: "Also rename FooTest?"
- Skip for now unless user demand emerges

## Priority Order

1 → 3 → 5 → 4 → 2 → 6

All items 1-5 implemented. Step 6 is deferred (low priority, exploratory).
