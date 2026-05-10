# v-018: Tab Close Behavior & Autosave

## Close Feedback Protocol

When a tab is asked to close (LRU eviction, `close` command, or max-tabs),
the tab gets a **feedback call** — it can accept or reject the close.

### Close Flow

```
1. Desktop calls view.can_close() → CloseResult
2. CloseResult::Ok → tab is removed
3. CloseResult::Denied(reason) → tab stays, reason shown in status bar
4. CloseResult::NeedsSave → autosave then close (or prompt if autosave off)
```

### Per-Tab-Type Behavior

| Type | On close request | On shell exit |
|------|-----------------|---------------|
| Editor (autosave on) | Save → close | N/A |
| Editor (autosave off, dirty) | Deny ("unsaved, use :w or :q!") | N/A |
| Editor (autosave off, clean) | Close | N/A |
| Shell (running) | Ask "kill process?" (deny for LRU) | Mark [exited], keep visible |
| Kiro (running) | Ask "end session?" (deny for LRU) | Mark [exited], keep visible |
| Find/Compile | Close + status message | N/A |

### LRU Eviction Rules

- NEVER evict a tab with a running process (Shell/Kiro) — skip to next LRU
- NEVER evict a dirty editor (autosave off) — skip to next LRU
- If ALL tabs would be denied, reject the new tab insertion

## Autosave

### Config

```tcl
# ~/.config/kairn/init.tcl
set autosave true        ;# default: true
set autosave_delay 5     ;# seconds of inactivity before save
```

### Behavior

- Timer starts on last edit (ContentChanged action)
- On timer fire: save silently (no status message unless error)
- On close: if dirty + autosave on → save immediately then close
- On close: if dirty + autosave off → deny close (use :w or :q!)
- Buffer marked clean after save (no re-trigger)

## Shell/Kiro Exit Detection

- PtyTerminal polls for output; when PTY EOF detected → mark as exited
- Title updates: `Shell:0` → `Shell:0 [exited]`
- Input disabled (keys ignored)
- Scrollback preserved (user can still read output)
- Tab can be closed freely (no feedback denial)

## Implementation Order (blockers for master merge)

1. Add `can_close() → CloseResult` to View trait (default: Ok)
2. Update LRU eviction to respect can_close (skip denied tabs)
3. Implement autosave (timer + save on ContentChanged inactivity)
4. Editor: implement can_close (save if autosave, deny if dirty)
5. Shell/Kiro: detect PTY exit, mark [exited], disable input
6. Shell/Kiro: implement can_close (deny if running, ok if exited)
7. Clipboard (OSC 52) — yank/paste to system clipboard

## Commands

| Command | Action |
|---------|--------|
| `close` | Request close on focused tab (respects feedback) |
| `close!` | Force close (kills process, discards changes) |
| `:q` | Same as `close` (editor) |
| `:q!` | Same as `close!` (editor) |
