# Context Broadcast + Tcl API

## Part 1: Context Broadcast

### Concept

The Program doesn't know about the status bar. Instead, it collects **context**
from the focused view and broadcasts it as a command every tick. Status bar items
(and any other interested view) listen for the context command and update themselves.

### Design

```rust
/// Context collected from the active view each tick.
pub struct ViewContext {
    pub file: Option<String>,       // current file path (relative)
    pub line: u32,                  // cursor line (1-indexed for display)
    pub col: u32,                   // cursor column
    pub mode: String,               // "NOR", "INS", "VIS", "CMD", etc.
    pub modified: bool,             // buffer dirty?
    pub language: String,           // "rust", "go", "python", ""
    pub encoding: String,           // "utf-8"
    pub line_ending: String,        // "LF", "CRLF"
    pub selection_lines: u32,       // 0 if no selection
    pub git_branch: String,         // current branch or ""
    pub lsp_status: String,         // "ready", "starting", "error", ""
    pub title: String,              // active tab title
}
```

### Flow

1. Every tick (or on focus change), the Program asks the focused view for context
2. Broadcast `CM_CONTEXT_UPDATE` with `Box<ViewContext>` 
3. Status bar items listen for `CM_CONTEXT_UPDATE` and update their labels
4. No coupling between Program and StatusBar internals

### Implementation

**Option A: View trait method**
```rust
trait View {
    fn context(&self) -> Option<ViewContext> { None }
}
```
EditorView returns file/line/col/mode/modified. Terminal returns title. Others return minimal.

**Option B: Collect from state**
The handler already has `AppState` with `cursor_pos`, `broker.last_opened()`, etc.
Just assemble ViewContext from state + desktop active tab info each tick.

**Recommendation: Option B** — simpler, no trait change, handler already has all the data.

### Status Bar Items That Use Context

| Item | Source field | Display |
|------|-------------|---------|
| Position | line, col | `Ln 42, Col 5` |
| Mode | mode | `NOR` / `INS` / `VIS` |
| File type | language | `rust` / `go` |
| Modified | modified | `[+]` or empty |
| Encoding | encoding | `UTF-8` |
| Line ending | line_ending | `LF` |
| Branch | git_branch | `main` |
| LSP | lsp_status | `◉` / `○` |
| Selection | selection_lines | `3 lines` |

### What Context Per View Type

| View | Context provided |
|------|-----------------|
| EditorView | file, line, col, mode, modified, language, encoding, line_ending, selection |
| Terminal | title (from OSC), mode="TERM" |
| ResultsView | title, line (cursor row), mode="RESULTS" |
| TreeView | title="Tree", mode="TREE" |
| TodoTree | title="Todo", mode="TODO" |
| Messages | title="Messages", mode="MSG" |
| HelpView | title="Help", mode="HELP" |

### Steps

1. Define `ViewContext` struct in `txv-core` (or kairn's commands module)
2. Add `CM_CONTEXT_UPDATE` command ID
3. In `handle_command`, on `CM_TICK`: assemble ViewContext from state + desktop, broadcast
4. Replace existing `CM_CURSOR_MOVED` / `CM_MODE_CHANGED` with single `CM_CONTEXT_UPDATE`
5. Update `PositionItem`, `ModeItem` to read from ViewContext
6. Add new items: FileType, Modified indicator, Encoding, Branch, LSP status
7. Remove the individual update commands (simplification)

---

## Part 2: Tcl API Implementation

### Prerequisites
- Rusticle interpreter embedded in kairn
- Bridge layer: Rust functions registered as Tcl procs

### Embedding Rusticle

```rust
// src/scripting/mod.rs
pub struct ScriptEngine {
    interp: rusticle::Interpreter,
}

impl ScriptEngine {
    pub fn new() -> Self {
        let mut interp = rusticle::Interpreter::new();
        // Register all built-in commands
        register_editor_commands(&mut interp);
        register_view_commands(&mut interp);
        register_system_commands(&mut interp);
        register_build_commands(&mut interp);
        register_lsp_commands(&mut interp);
        register_keymap_commands(&mut interp);
        register_hook_commands(&mut interp);
        Self { interp }
    }

    pub fn eval(&mut self, script: &str) -> Result<String, String> {
        self.interp.eval(script).map_err(|e| e.to_string())
    }

    pub fn load_file(&mut self, path: &Path) -> Result<(), String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        self.eval(&content)?;
        Ok(())
    }
}
```

### Bridge Pattern

Each Tcl command is a Rust closure that:
1. Receives args as `&[&str]`
2. Accesses kairn state via a shared reference (Arc<Mutex<AppState>> or similar)
3. Returns `Result<String, String>`

Problem: Tcl procs need access to AppState and EventQueue, but those are owned by the event loop.

**Solution: Command Queue**

Tcl procs don't execute actions directly. They push commands to a queue that the handler drains:

```rust
pub struct ScriptEngine {
    interp: rusticle::Interpreter,
    pending_commands: Vec<ScriptCommand>,
    // Read-only snapshot of state for queries
    state_snapshot: Arc<Mutex<StateSnapshot>>,
}

enum ScriptCommand {
    OpenFile { path: PathBuf, line: Option<u32> },
    SaveFile,
    GotoLine { line: u32, col: u32 },
    ShowMessage { level: MsgLevel, origin: String, text: String },
    RunBuild { command: String },
    SetKeyBinding { key: String, command: String },
    // ... etc
}
```

For **read** operations (editor current-line, current-file, etc.): use a state snapshot
updated each tick. Tcl reads from snapshot, no locking issues.

For **write** operations (editor insert, goto, save): push to pending_commands,
handler drains and executes after Tcl returns.

### Tcl API — Full Command List

#### `editor` namespace
```
editor open <path> ?-line N? ?-col N?
editor save
editor save-as <path>
editor save-all
editor close
editor reload
editor goto <line> ?<col>?
editor insert <text>
editor delete-line
editor delete-selection
editor undo
editor redo
editor select-all
editor select-line <n>
editor selection                    ;# returns selected text
editor current-file                 ;# returns path
editor current-line                 ;# returns line number (1-indexed)
editor current-col                  ;# returns column
editor line-count                   ;# returns total lines
editor get-line <n>                 ;# returns line content
editor word-at-cursor               ;# returns word
editor modified?                    ;# returns 0/1
editor set-option <key> <value>     ;# indent-width, use-tabs, etc.
editor filetype                     ;# returns language id
```

#### `buffer` namespace
```
buffer get <start> <end>            ;# returns lines as list
buffer replace <start> <end> <text>
buffer indent <start> <end>
buffer outdent <start> <end>
buffer comment <start> <end>        ;# toggle line comments
buffer sort <start> <end>           ;# sort lines
buffer unique <start> <end>         ;# remove duplicate lines
```

#### `view` namespace
```
view focus <slot>                   ;# left/center/right/bottom
view open-tab <slot> <title> <type> ?args?
view close-tab ?<title>?
view message <level> <origin> <text>  ;# error/warn/info
view status <text>                  ;# flash in status bar
view prompt <message> ?-default v?  ;# returns user input
view confirm <message>              ;# returns 0/1
view results <title> <entries>      ;# open results panel
view notify <text>                  ;# non-blocking notification
```

#### `build` namespace
```
build run ?<command>?               ;# run build (auto-detect if no arg)
build test ?<command>?              ;# run tests
build test-file                     ;# test current file
build test-at-cursor                ;# test function at cursor
build set-command <cmd>             ;# override build command
build set-test-command <cmd>        ;# override test command
build set-parser <proc-name>        ;# custom error parser
build detect                        ;# returns detected build command
build errors                        ;# returns current error list
build next-error                    ;# navigate to next
build prev-error                    ;# navigate to previous
```

#### `lsp` namespace
```
lsp hover                           ;# show hover popup
lsp definition                      ;# goto definition
lsp references                      ;# find references
lsp rename <new-name>               ;# rename symbol
lsp code-action                     ;# show code actions
lsp format                          ;# format document
lsp format-selection                ;# format selection
lsp diagnostics ?<file>?            ;# return diagnostics list
lsp status                          ;# return "ready"/"starting"/"error"/""
lsp restart                         ;# restart LSP server
```

#### `system` namespace
```
system exec <command>               ;# synchronous, returns stdout
system exec-async <command> ?-on-line proc? ?-on-done proc?
system env <var>                    ;# get env variable
system set-env <var> <value>        ;# set env variable
system root-dir                     ;# workspace root path
system home-dir                     ;# user home
system platform                     ;# "macos"/"linux"
system clipboard-get                ;# read clipboard
system clipboard-set <text>         ;# write clipboard
system tempfile ?<prefix>?          ;# create temp file, return path
```

#### `keymap` namespace
```
keymap bind <key> <command>         ;# bind key to command name
keymap unbind <key>                 ;# remove binding
keymap mode <name>                  ;# switch active mode
keymap define-mode <name> ?<parent>?  ;# create mode
keymap current-mode                 ;# return current mode name
keymap list-bindings ?<mode>?       ;# return all bindings
```

#### `hook` namespace
```
hook add <event> <proc-body>        ;# register hook
hook remove <event> ?<proc-body>?   ;# unregister (all if no body)
hook list ?<event>?                 ;# list registered hooks
hook fire <event> ?args...?         ;# manually fire (for testing)
```

Events: `file-open`, `file-save`, `file-close`, `buffer-modified`,
`cursor-moved`, `mode-changed`, `build-done`, `build-error`,
`tab-switched`, `startup`, `shutdown`, `todo-completed`, `todo-created`,
`lsp-ready`, `lsp-diagnostic`, `git-changed`

#### `grep` namespace
```
grep search <pattern> ?-root dir?   ;# async grep, opens results
grep search-literal <text>          ;# literal (no regex)
grep search-word                    ;# grep word at cursor
```

#### `git` namespace
```
git branch                          ;# current branch name
git status                          ;# return file status list
git diff ?<file>?                   ;# show diff
git commit <message>                ;# commit staged
git stage <file>                    ;# stage file
git unstage <file>                  ;# unstage file
git log ?-n N?                      ;# return log entries
```

#### `todo` namespace
```
todo add <text> ?-parent path?      ;# add item
todo remove <path>                  ;# remove item
todo complete <path>                ;# mark done
todo uncomplete <path>              ;# mark undone
todo set-priority <path> <level>    ;# high/normal/low
todo set-note <path> <text>         ;# set note
todo get <path>                     ;# return item dict
todo children <path>                ;# return child paths
todo move-up <path>                 ;# swap up
todo move-down <path>               ;# swap down
todo promote <path>                 ;# outdent
todo demote <path>                  ;# indent
```

#### `plugin` namespace
```
plugin register <name> <opts-dict>
plugin list
plugin enabled? <name>
plugin disable <name>
plugin enable <name>
```

#### `statusbar` namespace
```
statusbar add <id> <opts-dict>      ;# add custom item
statusbar remove <id>               ;# remove item
statusbar update <id>               ;# force refresh
```

### Config Loading

```
1. ~/.kairn/keymap.tcl        (or vim.tcl/vscode.tcl/emacs.tcl based on setting)
2. ~/.kairn/config.tcl        (user preferences, custom commands)
3. ~/.kairn/plugins/*/init.tcl (plugins, alphabetical)
4. .kairn/init.tcl            (project-specific, overrides)
```

### Error Handling

- ALL Tcl eval errors → `view message error "tcl" $error_message`
- Plugin init failures → logged + shown in messages, plugin disabled
- Hook failures → logged, hook continues (don't break other hooks)
- Never silent. Never crash kairn.

### Steps

1. Add `rusticle` as dependency (it's already in the workspace)
2. Create `src/scripting/mod.rs` with `ScriptEngine`
3. Create `src/scripting/bridge_editor.rs` — register editor commands
4. Create `src/scripting/bridge_view.rs` — register view commands
5. Create `src/scripting/bridge_system.rs` — register system commands
6. Create `src/scripting/bridge_build.rs` — register build commands
7. Create `src/scripting/bridge_keymap.rs` — register keymap commands
8. Create `src/scripting/bridge_hook.rs` — register hook commands
9. Create `src/scripting/bridge_lsp.rs` — register lsp commands
10. Create `src/scripting/bridge_git.rs` — register git commands
11. Create `src/scripting/bridge_todo.rs` — register todo commands
12. Wire into AppState: `pub script: ScriptEngine`
13. M-x / `:` command line → `script.eval(input)`
14. Load config files at startup
15. Fire hooks at appropriate points in handler

### Testing

- Unit test each bridge module: call Tcl proc, verify ScriptCommand produced
- Integration test: load a config.tcl, verify keybindings applied
- Integration test: hook fires on file-save
- Integration test: plugin registers and its commands work
