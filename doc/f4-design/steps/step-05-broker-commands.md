# Step 05: Open File Broker + Command Mode (M-x)

**Reference**: `doc/f4-design/v-012-view-hierarchy.md`, `doc/f4-design/v-013-txv-architecture.md`
**Depends on**: Steps 01-04 complete

## What this is

Two features that make kairn usable:
1. **Open File Broker** — tracks open files, prevents duplicates, coordinates tree↔editor
2. **Command Mode** — M-x prompt for application-level commands with tab completion

## Boundary

- **Modifies**: `src/` (kairn)
- **May add to**: `txv-widgets/` (if Completer trait needed there)
- **Does NOT touch**: txv-core/, txv-render/, rusticle/

## Part 1: Open File Broker

### What it does

A non-visual service in the App that tracks open files:

```rust
pub struct FileBroker {
    /// Currently open files: path → (slot, tab_index)
    open_files: HashMap<String, (SlotId, usize)>,
}

impl FileBroker {
    /// Try to open a file. Returns:
    /// - AlreadyOpen(slot, tab) if already open (just focus it)
    /// - Opened if newly opened
    pub fn open(&mut self, path: &str, slot: SlotId, tab: usize) -> OpenResult;

    /// Mark a file as closed.
    pub fn close(&mut self, path: &str);

    /// Check if a file is open.
    pub fn is_open(&self, path: &str) -> bool;

    /// Get all open file paths (for tree highlighting).
    pub fn open_paths(&self) -> Vec<&str>;
}

pub enum OpenResult {
    AlreadyOpen { slot: SlotId, tab: usize },
    Opened,
}
```

### How it integrates

In App's command handling:
```rust
CM_OPEN_FILE => {
    let path = payload.path;
    match self.broker.open(&path, SlotId::Center, next_tab) {
        OpenResult::AlreadyOpen { slot, tab } => {
            // Just focus that tab
            queue.put_command(CM_FOCUS_SLOT(slot), None);
            queue.put_command(CM_TAB_SWITCH(tab), None);
        }
        OpenResult::Opened => {
            // Create editor, insert in center
            let editor = EditorView::open(&path);
            self.desktop.insert_view(SlotId::Center, &title, editor);
        }
    }
}
```

### Tree highlighting

Tree can query broker (via a command or shared ref) to know which files
are open. Open files render with a different color (e.g., bold or green).

Command: `CM_QUERY_OPEN_FILES` → response with list. Or simpler: App
passes the open file set to tree on each draw cycle via a method.

### File deletion notification

When tree deletes a file:
1. Tree emits `CM_FILE_DELETED { path }`
2. App handles it: `broker.close(&path)` + close the editor tab

## Part 2: Command Mode (M-x)

### Trigger

StatusBar gets a new StatusItem:
```
Alt-x → CM_COMMAND_MODE, label: "M-x"
```

### What happens

1. StatusBar receives CM_COMMAND_MODE
2. StatusBar shows an InputLine prompt (replaces the label area temporarily)
3. User types a command with tab completion
4. Enter evaluates via rusticle interpreter
5. Esc cancels, restores normal status bar

### Completion system

```rust
/// Trait for providing completions.
pub trait Completer: Send {
    fn complete(&self, input: &str, cursor: usize) -> Vec<Completion>;
}

pub struct Completion {
    pub text: String,       // what to insert
    pub display: String,    // what to show in popup
    pub kind: &'static str, // "command", "file", "option", etc.
}
```

The command mode completer combines:
- Rusticle commands (from interpreter's command registry)
- File paths (from filesystem)
- Open buffer names (from broker)
- Options (from config declarations)

### Available commands

```
:help                    — show help in center slot
:about                   — show about info
:quit                    — quit
:open <path>             — open file (with file path completion)
:save                    — save current buffer
:close                   — close current tab
:shell                   — new shell tab in tools slot
:kiro [prompt]           — send to kiro
:compile                 — run build command
:test                    — run tests
:find <pattern>          — find in files (results in bottom slot)
:replace <pat> <rep>     — replace in files
:next-error              — jump to next error
:prev-error              — jump to previous error
:list-errors             — show error list
:set <option> <value>    — set config option
:bind <key> <command>    — add keybinding
:theme <name>            — switch theme
```

Each command is a rusticle proc registered in the bridge. Tab completion
comes from the rusticle command registry + file system.

### InputLine in StatusBar

StatusBar needs a mode: Normal (show labels) vs Prompt (show InputLine).
When in Prompt mode:
- StatusBar still has preprocess:true
- But it renders the InputLine instead of labels
- Keys go to InputLine (typing, completion, Enter, Esc)
- On Enter: evaluate input, emit result as command, return to Normal
- On Esc: return to Normal

## Part 3: Completion in vim : mode

The editor's `:` command mode should also use the Completer trait.
When user types `:` in vim normal mode:
- Editor shows its own command line (in the editor view, bottom row)
- Tab triggers completion (file paths for `:e`, commands for others)
- Same Completer infrastructure, different completers

This is a view-internal concern — the editor handles it. But it uses
the same Completer trait from txv-widgets (or txv-core).

## Where Completer lives

`txv-core` — it's a trait that any view can use. No I/O dependency.

```rust
// In txv-core/src/complete.rs
pub trait Completer: Send {
    fn complete(&self, input: &str, cursor: usize) -> Vec<Completion>;
}

pub struct Completion {
    pub text: String,
    pub display: String,
    pub kind: &'static str,
}
```

Concrete completers (file paths, commands, etc.) live in kairn.

## Verification

```bash
cargo build -p kairn
cargo test -p kairn
# Manual: open kairn, press M-x, type "help", press Tab, press Enter
# Manual: open same file twice from tree — should focus existing tab
```

## Do NOT

- Do NOT put broker logic in views (it's App-level)
- Do NOT make views query broker directly (use commands)
- Do NOT hardcode commands in the prompt (use rusticle registry)
- Do NOT skip completion (it's essential for usability)
- Do NOT make command mode modal (it's just a StatusBar mode switch)
