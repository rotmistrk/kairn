# Step 04: kairn

**Reference**: `doc/f4-design/v-012-view-hierarchy.md`, `doc/f4-design/v-013-txv-architecture.md`
**Depends on**: Steps 01-03 (txv-core, txv-render, txv-widgets)

## What this is

The IDE application. Composes views into a SlottedDesktop, wires commands.

## Boundary

- **Creates/modifies**: `src/` (kairn binary)
- **Does NOT touch**: txv-core/, txv-render/, txv-widgets/, rusticle/
- **Dependencies**: txv-core, txv-render, txv-widgets, rusticle, anyhow, clap, portable-pty, gix, nucleo, ignore, regex, similar, serde, serde_json, tokio

## Deliverables

```
src/
├── main.rs             — CLI (clap), create CrosstermBackend, create App, call txv_core::run()
├── commands.rs         — kairn-specific CommandIds (CM_OPEN_FILE, CM_SAVE, CM_NEW_SHELL, CM_ZOOM_TOGGLE, CM_FOCUS_LEFT/CENTER/RIGHT/BOTTOM, CM_TAB_NEXT/PREV, CM_SHOW_HELP, etc.)
├── desktop.rs          — SlottedDesktop (embeds GroupState, 4 slots with tabs, chrome, zoom)
├── status.rs           — KairnStatusBar (wraps txv_widgets::StatusBar, adds kairn-specific items)
├── views/
│   ├── mod.rs
│   ├── tree.rs         — FileTreeView (wraps TreeView<FileTreeData>)
│   ├── editor.rs       — EditorView (wraps Editor from src/editor/)
│   └── terminal.rs     — TerminalView (wraps TermBuf + PTY)
├── app.rs              — App: creates desktop + status bar, handles CM_OPEN_FILE/CM_QUIT/CM_SHOW_HELP
│
│   ── Pure logic modules (recovered from git, no UI deps) ──
├── buffer/             — PieceTable, LineIndex, UndoHistory
├── editor/             — Editor, Command, Keymap trait, VimKeymap, ExParser, save
├── kiro/               — KiroPayload, diff detection
├── lsp/                — LspClient, protocol, transport
├── nav/                — ImportIndex, Java/Go/Rust/TS navigators
├── content_search/     — workspace grep
├── git/                — gix operations
├── config/             — rusticle config loading, keybindings, themes
├── runner/             — build/test runners
└── session/            — session persistence
```

## Architecture rules

### App (~100 lines)
- Creates SlottedDesktop with initial views (tree in left, shell in right)
- Creates StatusBar with key bindings (F1-F5, Ctrl-Q)
- Inserts both into a root Group
- Calls `txv_core::run(&mut root, &mut backend)`
- Handles commands by creating views and inserting them into desktop

### SlottedDesktop (~300 lines)
- Embeds GroupState (gets three-phase dispatch for free)
- 4 named slots: Left, Center, Right, Bottom
- Each slot: `Vec<(String, Box<dyn View>)>` (title + view)
- Draws chrome: top line with tabs, vertical dividers, bottom divider
- Handles: CM_FOCUS_*, CM_TAB_*, CM_ZOOM_*, CM_SLOT_GROW/SHRINK
- Zoom: focused slot fills all space

### Views (~100-200 lines each)
- Each embeds ViewState, uses delegate_view_state!
- FileTreeView: wraps TreeView, on Enter calls queue.put_command(CM_OPEN_FILE, path)
- EditorView: wraps Editor, translates keys via keymap, calls queue.put_command for save/close
- TerminalView: wraps TermBuf + PTY, forwards keys

### StatusBar
- preprocess: true — sees keys before anyone
- StatusItems: F1→CM_SHOW_HELP, F2→CM_FOCUS_LEFT, F3→CM_FOCUS_CENTER, F4→CM_FOCUS_RIGHT, F5→CM_ZOOM_TOGGLE, Ctrl-Q→CM_QUIT
- Draws labels + context info

## Command flow

```
User presses Enter on tree
→ StatusBar sees it (preprocess), doesn't match any StatusItem, Ignored
→ SlottedDesktop dispatches to focused slot (left)
→ FileTreeView handles Enter, calls queue.put_command(CM_OPEN_FILE, path)
→ Run loop drains queue, dispatches CM_OPEN_FILE
→ App handles CM_OPEN_FILE: creates EditorView, inserts in center slot
```

## Recovering pure-logic modules

From git commit 4fca607. These modules have NO UI dependencies:
```bash
git show 4fca607:src/buffer/piece_table.rs > src/buffer/piece_table.rs
# ... etc (full list in v-013)
```

Remove any `mod render` or ratatui imports. These modules depend only on std.

## Verification

```bash
cargo build -p kairn
cargo clippy -p kairn -- -D warnings
dupfinder src/                  # no duplicates
kairn.f4                        # runs, shows tree, F-keys work, Enter opens files
```

## Do NOT

- Do NOT reach into desktop internals from App (send commands)
- Do NOT reach into view internals from desktop (it holds Box<dyn View>)
- Do NOT handle raw keys in App (StatusBar does that)
- Do NOT duplicate ViewState boilerplate (use delegate macro)
- Do NOT create outboxes, Arc<Mutex<>>, or any shared mutable state between views
- Do NOT skip select/unselect (focus must be visible to user)
- Do NOT redraw every frame (respect needs_redraw)
